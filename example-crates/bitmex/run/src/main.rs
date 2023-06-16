use anyhow::Context;
use bitmex_guest::{carol_activate, time, AttestIndexPrice, OffsetDateTime};
use carol_bls as bls;
use carol_host::{Executor, State};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = State::new(
        Executor::default(),
        bls::KeyPair::random(&mut rand::thread_rng()),
    );
    let binary = state
        .executor()
        .load_binary_from_wasm_file("target/bitmex_guest.wasm")
        .context("loading binary. You must compile it to target/bitmex_guest.wam first!")?;
    let method = carol_activate::AttestToPriceAtMinute {
        time: OffsetDateTime(time::OffsetDateTime::now_utc()),
        symbol: ".BXBT".into(),
    };

    let call = bincode::encode_to_vec(
        carol_activate::Activate::AttestToPriceAtMinute(method),
        bincode::config::standard(),
    )
    .unwrap();
    let output = state
        .executor()
        .activate_machine(state.clone(), &binary, &[], &call)
        .await??;

    let result = bincode::decode_from_slice::<Result<AttestIndexPrice<bls::Signature>, String>, _>(
        &output,
        bincode::config::standard(),
    )?
    .0;

    println!("{:?}", result);

    Ok(())
}
