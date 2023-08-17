use anyhow::{anyhow, Context};
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::Message;
use carol_core::{BinaryId, MachineId};
use carol_host::{CompiledBinary, Executor};
use clap::{Args, Parser, Subcommand};
use clap_cargo::Workspace;
use std::process::{Command, Stdio};
use wit_component::ComponentEncoder;

mod client;
use client::Client;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // TODO return a proper enum, of output types, and serialize them in a
    // consistent manner. for only paths and SHA256 hashes are output.
    match &cli.command {
        Commands::Build(opts) => println!("{}", opts.run(&Executor::new())?.0),
        Commands::Upload(opts) => {
            let server_opt = &opts.server;
            let binary_id = opts.run(&Executor::new(), &server_opt.new_client())?;
            if cli.quiet {
                println!("{}", binary_id)
            } else {
                println!(
                    "{}",
                    server_opt.url_for(&format!("/binaries/{}", binary_id))
                );
            }
        }
        Commands::Create(opts) => {
            let server_opt = &opts.implied_upload.server;
            let machine_id = opts.run(&Executor::new(), &server_opt.new_client())?;
            if cli.quiet {
                println!("{}", machine_id);
            } else {
                println!(
                    "{}",
                    server_opt.url_for(&format!("/machines/{}", machine_id))
                );
            }
        }
        Commands::Api(opts) => {
            let activations = opts.run(&Executor::new())?;
            println!("{}", activations.join("\n"));
        }
    };

    Ok(())
}

/// carlo: command line interface for Carol
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Write minial representation of output to stdout
    /// e.g. instead of outputing the full url to the resource just output the id.
    #[clap(short, long)]
    quiet: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build(BuildOpts),
    Upload(UploadOpts),
    Create(CreateOpts),
    Api(ApiOpts),
}

/// Inspect
#[derive(Args, Debug)]
pub struct ApiOpts {
    #[clap(flatten)]
    implied_build: BuildOpts,
    #[arg(
        long,
        value_name = "WASM_FILE",
        group = "api",
        conflicts_with = "build"
    )]
    binary: Option<Utf8PathBuf>,
}

#[derive(Args, Debug)]
/// Compile a Carol WASM component binary from a Rust crate
struct BuildOpts {
    #[arg(short, long, value_name = "SPEC", group = "build")]
    /// Package to compile to a Carol WASM component (see `cargo help pkgid`)
    pub package: Option<String>, // real one has Vec<String>
}

#[derive(Args, Debug)]
/// Upload a component binary to a Carol server
struct UploadOpts {
    /// The binary (WASM component) to upload (implied by --package)
    #[arg(
        long,
        value_name = "WASM_FILE",
        group = "upload",
        conflicts_with = "build"
    )]
    binary: Option<Utf8PathBuf>,

    #[clap(flatten)]
    implied_build: BuildOpts,

    #[clap(flatten)]
    server: ServerOpts,
}

#[derive(Args, Debug)]
/// Create a machine from a component binary on a Carol server
struct CreateOpts {
    /// The ID of the compiled binary from which to create a machine (implied by --binary)
    #[arg(
        long,
        value_name = "BINARY-ID",
        group = "create",
        conflicts_with = "upload",
        conflicts_with = "build"
    )]
    binary_id: Option<BinaryId>,

    #[clap(flatten)]
    implied_upload: UploadOpts,
}

#[derive(Args, Debug)]
struct ServerOpts {
    /// The Carol server's URL (e.g. http://localhost:8000, see README.md)
    #[arg(long)] // , default_value = "http://localhost:8000")] ?
    carol_url: String,
}

impl ServerOpts {
    pub fn new_client(&self) -> Client {
        Client::new(self.carol_url.clone())
    }

    pub fn url_for(&self, path: &str) -> String {
        format!("{}{}", self.carol_url, path)
    }
}

impl BuildOpts {
    fn run(&self, exec: &Executor) -> anyhow::Result<(Utf8PathBuf, CompiledBinary)> {
        // Find the crate package to compile
        let metadata = cargo_metadata::MetadataCommand::new()
            .exec()
            .context("Couldn't build Carol WASM component")?;
        let mut ws = Workspace::default();
        ws.package = self.package.iter().cloned().collect();
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

        // Compile to WASM target
        // TODO use cargo::ops::compile instead of invoking cargo CLI?
        let mut cmd = Command::new("cargo");
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

        eprintln!("Running {:?}", cmd);

        let mut proc = cmd.spawn().context("Couldn't spawn cargo rustc")?;

        let reader = std::io::BufReader::new(proc.stdout.take().unwrap());
        let messages = cargo_metadata::Message::parse_stream(reader)
            .collect::<Result<Vec<_>, _>>()
            .context("Couldn't read cargo output")?;

        let output = proc
            .wait_with_output()
            .context("Couldn't read `cargo rustc` output")?;

        if !output.status.success() {
            return Err(anyhow!(
                "`cargo rustc` exited unsuccessfully ({})",
                output.status
            ));
        }

        // Find the last compiler artifact message
        let final_artifact_message = messages
            .into_iter()
            .rev()
            .find_map(|message| match message {
                Message::CompilerArtifact(artifact) => Some(artifact),
                _ => None,
            })
            .ok_or_else(|| {
                anyhow!("No compiler artifact messages in output, could not find wasm output file.")
            })?;

        if final_artifact_message.filenames.len() != 1 {
            return Err(anyhow!(
                "Expected a single wasm artifact in files, but got:\n{}",
                final_artifact_message
                    .filenames
                    .iter()
                    .enumerate()
                    .map(|(i, name)| format!("{}: {}", i, name))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        let final_wasm_artifact = final_artifact_message.filenames[0].clone();

        let component_target = append_to_basename(&final_wasm_artifact, "-component")?;

        // Encode the component and write artifcat
        let wasm = std::fs::read(&final_wasm_artifact).context(format!(
            "Couldn't read compiled WASM file {final_wasm_artifact}"
        ))?;

        let encoder = ComponentEncoder::default()
            .validate(true)
            .module(&wasm)
            .context(format!(
                "validating wasm while transforming {final_wasm_artifact} into a component"
            ))?;

        let bytes = encoder
            .encode()
            .context("Failed to encode a component from module")?;

        std::fs::write(&component_target, bytes)
            .context(format!("Couldn't write WASM component {component_target}"))?;

        // TODO remove or (after careful consideration) convert to a
        // warning before release, as this strongly assumes the client side
        // carlo binary and server side carol host exactly agree on the
        // definition of Executor::load_binary_from_wasm_file.
        let compiled = exec
            .load_binary_from_wasm_file(&component_target)
            .context(format!(
                "Compiled WASM component {component_target} was invalid"
            ))?;

        Ok((component_target, compiled))
    }
}

impl UploadOpts {
    fn run(&self, exec: &Executor, client: &Client) -> anyhow::Result<BinaryId> {
        let binary = match &self.binary {
            Some(binary) => binary.clone(),
            None => {
                self.implied_build
                    .run(exec)
                    .context("Failed to build crate for upload")?
                    .0
            }
        };

        // Validate and derive BinaryId
        let binary_id = exec
            .load_binary_from_wasm_file(&binary)
            .context("Couldn't load compiled binary")?
            .binary_id();

        let file =
            std::fs::File::open(&binary).context(format!("Couldn't read file {}", binary))?;

        let response = client.upload_binary(&binary_id, file)?;
        let binary_id = response.id;
        Ok(binary_id)
    }
}

impl CreateOpts {
    fn run(&self, exec: &Executor, client: &Client) -> anyhow::Result<MachineId> {
        let binary_id = match self.binary_id {
            Some(binary_id) => binary_id,
            None => self
                .implied_upload
                .run(exec, &client)
                .context("Failed to upload binary for machine creation")?,
        };

        let response = client.create_machine(&binary_id)?;
        let machine_id = response.id;
        Ok(machine_id)
    }
}

impl ApiOpts {
    fn run(&self, exec: &Executor) -> anyhow::Result<Vec<String>> {
        let compiled = match &self.binary {
            Some(binary) => exec.load_binary_from_wasm_file(&binary)?,
            None => {
                self.implied_build
                    .run(exec)
                    .context("Failed to build crate for upload")?
                    .1
            }
        };

        let binary_api =
            tokio::runtime::Runtime::new()?.block_on(exec.get_binary_api(&compiled))?;
        Ok(binary_api
            .activations
            .into_iter()
            .map(|activation| activation.name)
            .collect())
    }
}

/// Helper function for rewriting filenames while retaining extension
fn append_to_basename(path: &Utf8PathBuf, suffix: &str) -> anyhow::Result<Utf8PathBuf> {
    let ext = path
        .extension()
        .context("Expected path to contain an extension")?
        .to_string();

    let basename = path
        .file_stem()
        .context("Expected path to contain a file basename component")?;

    let mut path = path.clone();
    path.set_file_name(format!("{basename}{suffix}"));
    path.set_extension(ext);
    Ok(path)
}
