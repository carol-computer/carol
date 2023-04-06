use anyhow::Context;
use carol_core::FullActivation;
use hello_world::HelloWorldMethods;
use std::fs::File;
use std::io::Read;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(name = "carol url")]
    carol_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let file_name = "target/hello_world.wasm";
    let mut file = File::open(file_name).context(format!("unable to open {}", file_name))?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;

    let call = bincode::encode_to_vec(
        &HelloWorldMethods::Say {
            message: "world!!!".into(),
        },
        bincode::config::standard(),
    )
    .unwrap();

    let binary = FullActivation {
        binary: &buf,
        parameters: &[],
        activation_input: &call,
    };

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/activate", args.carol_url))
        .body(bincode::encode_to_vec(
            &binary,
            bincode::config::standard(),
        )?)
        .send()
        .await?;

    let body_bytes = res.bytes().await?;

    let (message, _): (&str, _) =
        bincode::borrow_decode_from_slice(body_bytes.as_ref(), bincode::config::standard())?;

    println!("got response: {}", message);

    Ok(())
}
