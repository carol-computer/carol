use anyhow::{anyhow, Context};
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::Message;
use carol_core::BinaryId;
use carol_host::Executor;
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
        Commands::Build(opts) => println!("{}", opts.run()?),
        Commands::Upload(opts) => println!("{}", opts.run()?),
        Commands::Create(opts) => println!("{}", opts.run()?),
    };

    Ok(())
}

/// carlo: command line interface for Carol
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build(BuildOpts),
    Upload(UploadOptsWrapper),
    Create(CreateOpts),
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
}

#[derive(Args, Debug)]
/// Create a machine from a component binary on a Carol server
struct CreateOpts {
    #[clap(flatten)]
    server: ServerOpts,

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

    /// output the url rather than the machine id
    #[clap(long, short)]
    url: bool,
}

#[derive(Args, Debug)]
struct ServerOpts {
    /// The Carol server's URL (e.g. http://localhost:8000, see README.md)
    #[arg(long)] // , default_value = "http://localhost:8000")] ?
    carol_url: String,
}

// This wrapper is here because otherwise UploadOpts and CreateOpts specify
// duplicate ServerOpts/carol URL, which shouldn't be global because it doesn't
// actually apply to all commands
#[derive(Args, Debug)]
struct UploadOptsWrapper {
    #[clap(flatten)]
    server: ServerOpts,

    #[clap(flatten)]
    internal: UploadOpts,
}

impl BuildOpts {
    fn run(&self) -> anyhow::Result<Utf8PathBuf> {
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

        let encoder = ComponentEncoder::default().validate(true).module(&wasm)?;

        let bytes = encoder
            .encode()
            .context("Failed to encode a component from module")?;

        std::fs::write(&component_target, bytes)
            .context(format!("Couldn't write WASM component {component_target}"))?;

        // TODO remove or (after careful consideration) convert to a
        // warning before release, as this strongly assumes the client side
        // carlo binary and server side carol host exactly agree on the
        // definition of Executor::load_binary_from_wasm_file.
        _ = Executor::new()
            .load_binary_from_wasm_file(&component_target)
            .context(format!(
                "Compiled WASM component {component_target} was invalid"
            ))?;

        Ok(component_target)
    }
}

impl UploadOptsWrapper {
    fn run(&self) -> anyhow::Result<BinaryId> {
        let client = Client::new(self.server.carol_url.clone());
        self.internal.run(&client)
    }
}

impl UploadOpts {
    fn run(&self, client: &Client) -> anyhow::Result<BinaryId> {
        let binary = match &self.binary {
            Some(binary) => binary.clone(),
            None => self
                .implied_build
                .run()
                .context("Failed to build crate for upload")?,
        };

        // Validate and derive BinaryId
        let binary_id = Executor::new()
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
    fn run(&self) -> anyhow::Result<String> {
        let client = Client::new(self.server.carol_url.clone());

        let binary_id = match self.binary_id {
            Some(binary_id) => binary_id,
            None => self
                .implied_upload
                .run(&client)
                .context("Failed to upload binary for machine creation")?,
        };

        let response = client.create_machine(&binary_id)?;
        let machine_id = response.id;

        if self.url {
            Ok(format!("{}/machines/{}", self.server.carol_url, machine_id))
        } else {
            Ok(machine_id.to_string())
        }
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
