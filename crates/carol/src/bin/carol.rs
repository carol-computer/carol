use anyhow::Context;
use carol::config::Config;
use clap::Parser;
use std::path::PathBuf;
use tracing::{event, Level};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, name = "yaml config file")]
    cfg: PathBuf,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config: Config = {
        use std::{fs::File, io::Read};
        let file_name = args.cfg.to_str().unwrap_or("config file").to_owned();
        let mut file = File::open(args.cfg)
            .context(format!("unable to open configuration file {}", file_name))?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        serde_yaml::from_str(&content)
            .context(format!("{} is an invalid configuration file", file_name))?
    };

    let subscriber = tracing_subscriber::fmt().with_max_level(Level::DEBUG).finish();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber)?;
    event!(Level::INFO, "starting carol");

    carol::http_server::start(config.http_server).await?;

    Ok(())
}
