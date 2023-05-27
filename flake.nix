{
  description = "Carol";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [
            # use rust overlay to provide a rust toolchain same as rustup would
            (import rust-overlay)
            (self: super: {
              rustToolchain = super.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
            })
          ];

          pkgs = import nixpkgs {
            inherit system overlays;
          };

          craneLib = crane.mkLib pkgs;

          src = pkgs.lib.cleanSourceWith {
            src = craneLib.path ./.;
            filter = path: type:
              (pkgs.lib.hasSuffix "\.wit" path) ||
              (pkgs.lib.hasSuffix "\.md" path) ||
              (craneLib.filterCargoSources path type);
          };

          # parse the list of workspace members, separating the example crates
          # out from the packages we want to output in the flake packages
          readTomlFile = path: builtins.fromTOML (builtins.readFile path);
          workspaceMembers = (readTomlFile ./Cargo.toml).workspace.members;
          isExample = p: (builtins.substring 0 15 p) == "example-crates/";
          exportedMembers = builtins.filter (x: !isExample x) workspaceMembers;
          exampleMembers = builtins.filter isExample workspaceMembers;

          packageMetadata = crateDir: craneLib.crateNameFromCargoToml {
            cargoToml = ./. + "/${crateDir}/Cargo.toml";
          };

          carolCrateMetadata = packageMetadata "crates/carol";
          exportedPackageMetadata = builtins.map packageMetadata exportedMembers;
          examplePackageMetadata = builtins.map packageMetadata exampleMembers;

          # derivation arguments for skipping normal binary installation
          # used for checks with no build outputs or build artifact caching
          # derivations which are not meant to package what is built
          skipInstallArgs = {
            installPhaseCommand = "";
            dontFixup = true;
          };

          # define an unexported package for caching the workspace build and its
          # dependencies, which will get reused in subsequent derivations
          cargoArtifacts =
            let
              pname = "carol-workspace";
            in
            craneLib.buildPackage (with carolCrateMetadata; skipInstallArgs // {
              # first, cache just the dependencies
              cargoArtifacts =
                craneLib.buildDepsOnly (with carolCrateMetadata; skipInstallArgs // {
                  inherit pname version src;
                });

              # next, build the workspace normally (no -p args to cargo build)
              inherit pname version src;
              pnameSuffix = "-artifacts";
              doInstallCargoArtifacts = true;
              doCheck = false; # don't fail just yet
            });

          # finally build again, one package at a time. this prevents parallel
          # re-compilation of the per-crate dependencies in the flake checks,
          # but doesn't really avoid the underlying cost of multiple compiler
          # invocations.
          #
          # quoting from https://doc.rust-lang.org/cargo/reference/resolver.html#features
          # > When building multiple packages in a workspace (such as with --workspace or
          # > multiple -p flags), the features of the dependencies of all of those
          # > packages are unified. If you have a circumstance where you want to avoid
          # > that unification for different workspace members, you will need to build
          # > them via separate cargo invocations.
          exhaustiveCargoArtifacts = craneLib.buildPackage (with carolCrateMetadata; skipInstallArgs // {
            inherit version src cargoArtifacts;
            pname = "carol-workspace";
            pnameSuffix = "-exhaustive-artifacts-hack";
            doCheck = false;
            doInstallCargoArtifacts = true;
            buildPhaseCargoCommand = ''
              for pkg in ${builtins.toString (builtins.map (p: p.pname) (exportedPackageMetadata ++ examplePackageMetadata))}; do
                cargoWithProfile build -p $pkg
              done
            '';
          });

          # helper function for providing a package definition given a
          # Cargo.toml in the workspace
          buildWorkspacePackage = cargoMeta:
            craneLib.buildPackage (cargoMeta // {
              inherit src;
              cargoArtifacts = exhaustiveCargoArtifacts;
              doCheck = false; # tests are run in nix flake check with cargo nextest
              cargoExtraArgs = "-p ${cargoMeta.pname}";
            });

          # these are nix packages, as opposed to crate packages
          exportedPackages = builtins.map buildWorkspacePackage exportedPackageMetadata;
          examplePackages = builtins.map buildWorkspacePackage examplePackageMetadata;

          # helper function to convert a list of packages to an attr set
          packagesToAttrs = list: builtins.listToAttrs (builtins.map
            (p: {
              name = p.pname;
              value = p;
            })
            list);

          carolToolchain = pkgs.buildEnv {
            name = "carol-toolchain";
            paths = with (packagesToAttrs exportedPackages); [ pkgs.rustToolchain carol ];
          };
          carolCrates = pkgs.buildEnv {
            name = "carol-crates";
            paths = exportedPackages;
          };
          examples = pkgs.buildEnv {
            name = "carol-examples";
            paths = examplePackages;
          };
        in
        with pkgs;
        {
          packages = (packagesToAttrs exportedPackages) // {
            default = carolToolchain;
            examples = examples // (packagesToAttrs examplePackages);
          };

          devShells.default = mkShell {
            buildInputs = [
              rustToolchain
              lldb
              cargo-nextest
              wasm-tools
            ];
          };
        }
      );
}
