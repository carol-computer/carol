use carol_host::Executor;
use hello_world::HelloWorldMethods;

fn main() -> anyhow::Result<()> {
    let exec = Executor::new();
    let contract = exec.load_contract_from_file("target/hello_world.wasm")?;

    let call = bincode::encode_to_vec(
        &HelloWorldMethods::Say {
            message: "world!!!".into(),
        },
        bincode::config::standard(),
    )
    .unwrap();

    exec.execute_contract(contract, vec![], call)?;

    Ok(())
}
