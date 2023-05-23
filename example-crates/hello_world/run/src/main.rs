use anyhow::Context;
use carol_bls as bls;
use carol_host::{Executor, State};
use hello_world::carol_activate;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = State::new(
        Executor::default(),
        bls::KeyPair::random(&mut rand::thread_rng()),
    );
    let binary = state
        .executor()
        .load_binary_from_wasm_file("target/hello_world.wasm")
        .context("target/hello_world.wasm not found. Make sure to build it first!")?;

    let call = bincode::encode_to_vec(
        carol_activate::Activate::Say(carol_activate::Say {
            message: "world!!!".into(),
        }),
        bincode::config::standard(),
    )
    .unwrap();

    let output = state
        .executor()
        .activate_machine(state.clone(), &binary, &[], &call)
        .await??;
    let (output, _): (&str, _) =
        bincode::borrow_decode_from_slice(&output, bincode::config::standard())?;
    println!("got response: {}", output);

    Ok(())
}
