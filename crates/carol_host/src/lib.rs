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
        config.wasm_component_model(true);
        let engine = Engine::new(&config).expect("valid config");

        Self { engine }
    }

    pub fn load_contract_from_file(&self, file: impl AsRef<Path>) -> anyhow::Result<Contract> {
        Ok(Contract {
            component: Component::from_file(&self.engine, file)?,
        })
    }

    pub fn execute_contract(
        &self,
        contract: Contract,
        contract_params: Vec<u8>,
        exec_args: Vec<u8>,
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
            },
        );
        let (bindings, _) = RunContract::instantiate(&mut store, &contract.component, &linker)?;

        // // Here our `greet` function doesn't take any parameters for the component,
        // // but in the Wasmtime embedding API the first argument is always a `Store`.
        let output = bindings
            .contract()
            .call_run(&mut store, &contract_params, &exec_args)?;

        Ok(output)
    }
}
