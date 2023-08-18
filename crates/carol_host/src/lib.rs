mod host_bindings;
mod state;
pub use state::*;

use anyhow::Context;
use carol_core::{hex, BinaryId, MachineId};
pub use host_bindings::guest;
use host_bindings::{Environment, Host, Machine};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::{event, info_span, Instrument, Level};
use wasmtime::{component::*, WasmBacktrace};
use wasmtime::{Config, Engine, Store};

#[derive(Clone)]
pub struct Executor {
    engine: Engine,
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct CompiledBinary {
    component: Component,
    binary_id: BinaryId,
}

impl CompiledBinary {
    pub fn binary_id(&self) -> BinaryId {
        self.binary_id
    }
}

#[derive(Debug)]
pub enum GuestError {
    Panic {
        backtrace: Option<WasmBacktrace>,
        message: String,
    },
    Other(anyhow::Error),
}

impl core::fmt::Display for GuestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuestError::Panic { backtrace, message } => {
                write!(f, "guest panic ‘{}’", message)?;
                if let Some(bt) = backtrace {
                    write!(f, "\n{}", bt)?;
                }
                Ok(())
            }
            GuestError::Other(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for GuestError {}

impl Executor {
    pub fn new() -> Self {
        let mut config = Config::new();
        config.async_support(true);
        config.wasm_component_model(true);
        let engine = Engine::new(&config).expect("valid config");

        Self { engine }
    }

    pub fn load_binary_from_wasm_file(
        &self,
        file: impl AsRef<Path>,
    ) -> anyhow::Result<CompiledBinary> {
        let mut f = File::open(file)?;
        let mut binary = vec![];
        f.read_to_end(&mut binary)?;
        self.load_binary_from_wasm_binary(&binary)
    }

    pub fn load_binary_from_wasm_binary(&self, binary: &[u8]) -> anyhow::Result<CompiledBinary> {
        let binary_id = BinaryId::new(binary);
        Ok(CompiledBinary {
            component: Component::from_binary(&self.engine, binary)?,
            binary_id,
        })
    }

    pub async fn get_binary_api(
        &self,
        compiled_binary: &CompiledBinary,
    ) -> anyhow::Result<host_bindings::guest::BinaryApi> {
        let dummy_host = Host {
            env: Environment::BinaryApi,
            panic_message: None,
        };

        let mut linker = Linker::new(&self.engine);
        Machine::add_to_linker(&mut linker, |state: &mut Host| state)?;

        let mut store = Store::new(&self.engine, dummy_host);

        let span = info_span!("describe_binary");

        let (bindings, _) =
            Machine::instantiate_async(&mut store, &compiled_binary.component, &linker).await?;

        let output = bindings
            .carol_machine_guest()
            .call_get_binary_api(&mut store)
            .instrument(span)
            .await?;

        Ok(output)
    }

    pub async fn activate_machine(
        &self,
        state: State,
        compiled_binary: &CompiledBinary,
        machine_params: &[u8],
        activation_name: &str,
        activation_input: &[u8],
    ) -> anyhow::Result<Result<Vec<u8>, GuestError>> {
        let machine_id = MachineId::new(compiled_binary.binary_id, machine_params);
        // Instantiation of bindings always happens through a `Linker`.
        // Configuration of the linker is done through a generated `add_to_linker`
        // method on the bindings structure.
        let mut linker = Linker::new(&self.engine);
        Machine::add_to_linker(&mut linker, |state: &mut Host| state)?;

        // // As with the core wasm API of Wasmtime instantiation occurs within a
        // // `Store`. The bindings structure contains an `instantiate` method which
        // // takes the store, component, and linker. This returns the `bindings`
        // // structure which is an instance of `HelloWorld` and supports typed access
        // // to the exports of the component.
        let mut store = Store::new(
            &self.engine,
            Host {
                env: Environment::Activation {
                    http_client: reqwest::Client::new(),
                    machine_id,
                    state,
                },
                panic_message: None,
            },
        );

        // struct Handler {}
        // #[async_trait]
        // impl CallHookHandler<Host> for Handler {
        //     async fn handle_call_event(&self, t: &mut Host, ch: wasmtime::CallHook) -> anyhow::Result<()> {
        //         dbg!(&ch);
        //         Ok(())
        //     }
        // }
        // store.call_hook_async(Handler {});
        let (bindings, _) =
            Machine::instantiate_async(&mut store, &compiled_binary.component, &linker).await?;
        {
            let params = hex::encode(&machine_params[..machine_params.len().min(8)]);
            let input = hex::encode(&activation_input[..activation_input.len().min(8)]);
            event!(
                Level::INFO,
                machine_id = machine_id.to_string(),
                params,
                input,
                "begin activation"
            )
        }
        let span = info_span!("activation", machine_id = machine_id.to_string());
        // // Here our `greet` function doesn't take any parameters for the component,
        // // but in the Wasmtime embedding API the first argument is always a `Store`.
        let output = bindings
            .carol_machine_guest()
            .call_activate(
                &mut store,
                machine_params,
                activation_name,
                activation_input,
            )
            .instrument(span)
            .await;

        match output {
            Ok(output) => Ok(Ok(output)),
            Err(e) => Ok(Err(match &store.data().panic_message {
                Some(message) => {
                    event!(Level::ERROR, message = message, "panic during activation");
                    let backtrace = e.downcast::<WasmBacktrace>().ok();
                    GuestError::Panic {
                        backtrace,
                        message: message.clone(),
                    }
                }
                None => GuestError::Other(e),
            })),
        }
    }

    pub async fn machine_handle_http_request(
        &self,
        state: State,
        compiled_binary: &CompiledBinary,
        machine_params: &[u8],
        mut req: http_crate::Request<hyper::Body>,
    ) -> anyhow::Result<Result<http_crate::Response<hyper::Body>, GuestError>> {
        use hyper::body::HttpBody;
        let machine_id = MachineId::new(compiled_binary.binary_id, machine_params);
        let mut linker = Linker::new(&self.engine);
        Machine::add_to_linker(&mut linker, |host: &mut Host| host)?;
        let mut store = Store::new(
            &self.engine,
            Host {
                env: Environment::Http { machine_id, state },
                panic_message: None,
            },
        );

        let (bindings, _) =
            Machine::instantiate_async(&mut store, &compiled_binary.component, &linker).await?;

        let body_stream = req.body_mut();
        let mut body = Vec::with_capacity(body_stream.size_hint().upper().unwrap_or(0) as usize);

        while let Some(chunk) = body_stream.data().await {
            let chunk = chunk.with_context(|| {
                format!("failed to finished reading http request to {}", machine_id)
            })?;
            body.extend_from_slice(chunk.as_ref());
        }

        let span = info_span!(
            "machine_handle_http_request",
            machine_id = machine_id.to_string()
        );
        let response = bindings
            .carol_machine_guest()
            .call_handle_http(
                &mut store,
                &host_bindings::http::Request {
                    method: req.method().clone().try_into()?,
                    uri: req.uri().to_string(),
                    headers: vec![], // TODO: support headers
                    body,
                },
            )
            .instrument(span.clone())
            .await
            .map(|response| response.try_into());

        let _enter = span.enter();

        match response {
            Ok(Ok(response)) => Ok(Ok(response)),
            Ok(Err(e)) => {
                event!(
                    Level::ERROR,
                    "the guest HTTP response couldn't be turned into a hyper::Response"
                );
                Ok(Err(GuestError::Other(e)))
            }
            Err(e) => Ok(Err(match &store.data().panic_message {
                Some(message) => {
                    event!(
                        Level::ERROR,
                        message = message,
                        "panic while handling http request"
                    );
                    let backtrace = e.downcast::<WasmBacktrace>().ok();
                    GuestError::Panic {
                        backtrace,
                        message: message.clone(),
                    }
                }
                None => {
                    event!(Level::ERROR, "guest other error: {}", e);
                    GuestError::Other(e)
                }
            })),
        }
    }
}
