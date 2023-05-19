use anyhow::{anyhow, Context};
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::Message;
use carol_core::BinaryId;
use carol_host::Executor;
use clap::{Parser, Subcommand};
use clap_cargo::Workspace;
use std::process::{Command, Stdio};
use wit_component::ComponentEncoder;

mod client;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build {
        #[arg(short, long, value_name = "SPEC", group = "build")]
        /// Package to compile to a Carol WASM component (see `cargo help pkgid`)
        package: Option<String>, // real one has Vec<String>
    },
    Upload {
        #[arg(long)]
        carol_url: String,
        #[arg(long)]
        binary: Utf8PathBuf,
    },
    Create {
        #[arg(long)]
        carol_url: String,
        #[arg(long)]
        binary_id: BinaryId,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Build { package } => {
            let mut cmd = Command::new("cargo");

            // Find the crate package to compile
            let metadata = cargo_metadata::MetadataCommand::new()
                .exec()
                .context("Couldn't build Carol WASM component")?;

            let mut ws = Workspace::default();
            ws.package = package.iter().cloned().collect();
            let (included, _) = ws.partition_packages(&metadata);
            if included.is_empty() {
                return Err(anyhow!(
                    "package ID specification {:?} did not match any packages",
                    ws.package
                ));
            }
            if included.len() != 1 {
                return Err(anyhow!("Carol WASM components must be built from a single crate, but package ID specification {:?} resulted in {} packages (did you forget to specify -p in a workspace?)", ws.package, included.len()));
            }
            let package = &included[0].name;

            cmd.env("RUSTFLAGS", "-C opt-level=z")
                .args([
                    "rustc",
                    "--package",
                    package,
                    "--message-format=json-render-diagnostics",
                    "--target",
                    "wasm32-unknown-unknown",
                    "--release",
                    "--crate-type=cdylib",
                ])
                .stdout(Stdio::piped());

            let mut proc = cmd.spawn().expect("things to compile");

            let reader = std::io::BufReader::new(proc.stdout.take().unwrap());

            let messages = cargo_metadata::Message::parse_stream(reader)
                .collect::<Result<Vec<_>, _>>()
                .context("Reading cargo output")?;

            let final_artifact_message = messages
                .into_iter()
                .rev()
                .find_map(|message| match message {
                    Message::CompilerArtifact(artifact) => Some(artifact),
                    _ => None,
                })
                .ok_or(anyhow!(
                    "No compiler artifact messages in output, could not find wasm output file"
                ))?;

            if final_artifact_message.filenames.len() != 1 {
                return Err(anyhow!(
                    "Expected a single wasm artifact in files, but got {}",
                    final_artifact_message.filenames.len()
                ));
            }

            let final_wasm_artifact = final_artifact_message.filenames[0].clone();

            proc.wait().expect("Couldn't get cargo's exit status");

            let mut component_target = final_wasm_artifact.clone();
            // FIXME horrible jankyness, but it works
            component_target.set_extension("");
            component_target
                .set_file_name(component_target.file_name().unwrap().to_owned() + "-component");
            component_target.set_extension("wasm");

            let wasm = std::fs::read(&final_wasm_artifact)
                .context(format!("Reading compiled WASM file {final_wasm_artifact}"))?;

            let encoder = ComponentEncoder::default().validate(true).module(&wasm)?;

            let bytes = encoder
                .encode()
                .context("failed to encode a component from module")?;

            std::fs::write(&component_target, bytes)
                .context(format!("Writing WASM component {component_target}"))?;

            // TODO remove or (after careful consideration) convert to a
            // warning before release, as this strongly assumes the client side
            // carlo binary and server side carol host exactly agree on the
            // definition of Executor::load_binary_from_wasm_file.
            _ = Executor::new()
                .load_binary_from_wasm_file(&component_target)
                .context("Ensuring WASM component {component_target} is loadable")?;

            println!("{component_target}");
        }
        Commands::Upload { binary, carol_url } => {
            let client = client::Client::new(carol_url.clone());

            // Validate and derive BinaryId
            let binary_id = Executor::new()
                .load_binary_from_wasm_file(binary)
                .context("Loading compiled binary")?
                .binary_id();

            let file = std::fs::File::open(binary)
                .context(format!("Reading compiled WASM file {}", binary))?;

            let response = client.upload_binary(&binary_id, file)?;
            let binary_id = response.id;
            println!("{binary_id}");
        }
        Commands::Create {
            binary_id,
            carol_url,
        } => {
            let client = client::Client::new(carol_url.clone());

            let response = client.create_machine(binary_id)?;
            let machine_id = response.id;
            print!("{machine_id}");
        }
    }

    Ok(())
}
