use std::path::Path;
use carol_core::MachineId;
use sha2::{Sha256, Digest};
use tracing::{info_span, Instrument};
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
mod host_bindings;
use host_bindings::{BlsKeyPair, Host, Machine};
use std::fs::File;
use std::io::Read;

#[derive(Clone)]
pub struct Executor {
    engine: Engine,
}

#[derive(Clone)]
pub struct CompiledMachine {
    component: Component,
    binary_hash: [u8;32],
}

impl Executor {
    pub fn new() -> Self {
        let mut config = Config::new();
        config.async_support(true);
        config.wasm_component_model(true);
        let engine = Engine::new(&config).expect("valid config");

        Self { engine }
    }

    pub fn load_machine_from_wasm_file(&self, file: impl AsRef<Path>) -> anyhow::Result<CompiledMachine> {
        let mut f = File::open(file)?;
        let mut binary = vec![];
        f.read_to_end(&mut binary)?;
        self.load_machine_from_wasm_binary(&binary)
    }

    pub fn load_machine_from_wasm_binary(&self, binary: &[u8]) -> anyhow::Result<CompiledMachine> {
        let binary_hash = Sha256::default().chain(binary).finalize().into();
        Ok(CompiledMachine {
            component: Component::from_binary(&self.engine, binary)?,
            binary_hash,
        })
    }

    pub async fn activate_machine(
        &self,
        machine: CompiledMachine,
        machine_params: &[u8],
        activation_input: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        let machine_id = MachineId::new(machine.binary_hash, machine_params);
        // Instantiation of bindings always happens through a `Linker`.
        // Configuration of the linker is done through a generated `add_to_linker`
        // method on the bindings structure.
        //
        // Note that the closure provided here is a projection from `T` in
        // `Store<T>` to `&mut U` where `U` implements the `HelloWorldImports`
        // trait. In this case the `T`, `MyState`, is stored directly in the
        // structure so no projection is necessary here.
        let mut linker = Linker::new(&self.engine);
        Machine::add_to_linker(&mut linker, |state: &mut Host| state)?;

        struct Handler {}
        // // As with the core wasm API of Wasmtime instantiation occurs within a
        // // `Store`. The bindings structure contains an `instantiate` method which
        // // takes the store, component, and linker. This returns the `bindings`
        // // structure which is an instance of `HelloWorld` and supports typed access
        // // to the exports of the component.
        let mut store = Store::new(
            &self.engine,
            Host {
                bls_keypair: BlsKeyPair::random(&mut rand::thread_rng()),
                machine_id,
                http_client: reqwest::Client::new(),
            },
        );
        // #[async_trait]
        // impl CallHookHandler<Host> for Handler {
        //     async fn handle_call_event(&self, t: &mut Host, ch: wasmtime::CallHook) -> anyhow::Result<()> {
        //         dbg!(&ch);
        //         Ok(())
        //     }
        // }
        // store.call_hook_async(Handler {});
        let (bindings, _) =
            Machine::instantiate_async(&mut store, &machine.component, &linker).await?;

        let span = info_span!("activation", machine_id = machine_id.to_string());
        // // Here our `greet` function doesn't take any parameters for the component,
        // // but in the Wasmtime embedding API the first argument is always a `Store`.
        let output = bindings
            .machine()
            .call_activate(&mut store, &machine_params, &activation_input)
            .instrument(span)
            .await?;

        Ok(output)
    }
}
