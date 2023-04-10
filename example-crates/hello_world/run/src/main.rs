use carol_host::Executor;
use hello_world::HelloWorldMethods;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let exec = Executor::new();
    let machine = exec.load_machine_from_wasm_file("target/hello_world.wasm")?;

    let call = bincode::encode_to_vec(
        &HelloWorldMethods::Say {
            message: "world!!!".into(),
        },
        bincode::config::standard(),
    )
    .unwrap();

    let output = exec.activate_machine(machine, &[], &call).await?;
    let (output, _): (&str, _) =
        bincode::borrow_decode_from_slice(&output, bincode::config::standard())?;

    println!("got response: {}", output);

    Ok(())
}
