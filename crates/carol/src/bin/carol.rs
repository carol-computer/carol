use anyhow::{anyhow, Context};
use carol::config::Config;
use carol_host::{Executor, State};
use clap::{Parser, Subcommand};
use std::{fs::File, path::PathBuf};
use tracing::{event, Level};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long, name = "yaml config file")]
    cfg: PathBuf,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Generate a config file to the --cfg path
    ConfigGen,
    /// Run carol
    Run,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let file_name = args
        .cfg
        .to_str()
        .expect("able to get file name as string")
        .to_owned();
    let file_path = args.cfg;

    match args.command {
        Commands::Run => {
            let mut file = File::open(file_path).context(format!(
                "unable to open configuration file {file_name} for reading"
            ))?;

            let config: Config = {
                use std::io::Read;
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                serde_yaml::from_str(&content)
                    .context(format!("{file_name} is an invalid configuration file"))?
            };

            let subscriber = tracing_subscriber::fmt()
                .with_max_level(config.log.level)
                .pretty()
                .finish();
            // use that subscriber to process traces emitted after this point
            tracing::subscriber::set_global_default(subscriber)?;
            event!(Level::INFO, "starting carol");

            let state = State::new(Executor::default(), config.bls_secret_key);

            carol::http::server::start(config.http_server, state).await?;
        }
        Commands::ConfigGen => {
            if file_path.exists() {
                return Err(anyhow!(
                    "config file {file_name} already exists. Remove it to generate a new one."
                ));
            }
            let mut file = File::create(file_path).context("creating {file_name}")?;
            let config = Config::generate(&mut rand::thread_rng());
            serde_yaml::to_writer(&mut file, &config)
                .context(format!("writing newly generated config to {file_name}"))?;
            file.sync_all().context("syncing new config to file")?
        }
    }

    Ok(())
}
