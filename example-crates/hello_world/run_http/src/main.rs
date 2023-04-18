use anyhow::{anyhow, Context};
use hello_world::HelloWorldMethods;
use reqwest::header;
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
    let mut binary = vec![];
    file.read_to_end(&mut binary)?;

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/binaries", args.carol_url))
        .body(binary)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(anyhow!("{}", res.text().await?));
    }
    else {
        let location = res.headers().get(header::LOCATION);
        println!("posting binary succeeded {:?}", location);
    }


    let res = client.post(format!())

    Ok(())
}
