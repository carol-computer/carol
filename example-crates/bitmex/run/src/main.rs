use bitmex_guest::{
    time, AttestIndexPrice, BitMexAttest, BitMexAttestMethods, Index, OffsetDateTime,
};
use carol_host::Executor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let exec = Executor::new();
    let contract = exec.load_contract_from_file("target/bitmex_guest.wasm")?;
    let index = Index::BXBT;

    let instance =
        bincode::encode_to_vec(&BitMexAttest { index }, bincode::config::standard()).unwrap();

    let call = bincode::encode_to_vec(
        &BitMexAttestMethods::AttestToPriceAtMinute {
            time: OffsetDateTime(time::OffsetDateTime::now_utc()),
        },
        bincode::config::standard(),
    )
    .unwrap();
    let output = exec.execute_contract(contract, instance, call).await?;

    let result = bincode::decode_from_slice::<Result<AttestIndexPrice, String>, _>(
        &output,
        bincode::config::standard(),
    )?
    .0;

    println!("{:?}", result);

    Ok(())
}
