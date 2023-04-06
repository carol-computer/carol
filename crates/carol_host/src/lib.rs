use std::path::Path;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
mod host_bindings;

use host_bindings::{BlsKeyPair, Host, RunContract};

#[derive(Clone)]
pub struct Executor {
    engine: Engine,
}

#[derive(Clone)]
pub struct Contract {
    component: Component,
}

impl Executor {
    pub fn new() -> Self {
        let mut config = Config::new();
        config.async_support(true);
        config.wasm_component_model(true);
        let engine = Engine::new(&config).expect("valid config");

        Self { engine }
    }

    pub fn load_contract_from_file(&self, file: impl AsRef<Path>) -> anyhow::Result<Contract> {
        Ok(Contract {
            component: Component::from_file(&self.engine, file)?,
        })
    }

    pub fn load_contract_from_binary(&self, binary: &[u8]) -> anyhow::Result<Contract> {
        Ok(Contract {
            component: Component::from_binary(&self.engine, binary)?,
        })
    }

    pub async fn execute_contract(
        &self,
        contract: Contract,
        contract_params: &[u8],
        activation_input: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        // Instantiation of bindings always happens through a `Linker`.
        // Configuration of the linker is done through a generated `add_to_linker`
        // method on the bindings structure.
        //
        // Note that the closure provided here is a projection from `T` in
        // `Store<T>` to `&mut U` where `U` implements the `HelloWorldImports`
        // trait. In this case the `T`, `MyState`, is stored directly in the
        // structure so no projection is necessary here.
        let mut linker = Linker::new(&self.engine);
        RunContract::add_to_linker(&mut linker, |state: &mut Host| state)?;

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
                contract_id: [0u8; 32],
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
            RunContract::instantiate_async(&mut store, &contract.component, &linker).await?;

        // // Here our `greet` function doesn't take any parameters for the component,
        // // but in the Wasmtime embedding API the first argument is always a `Store`.
        let output = bindings
            .contract()
            .call_activate(&mut store, &contract_params, &activation_input)
            .await?;

        Ok(output)
    }
}
