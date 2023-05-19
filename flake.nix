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

          carolCrateMetadata = craneLib.crateNameFromCargoToml { cargoToml = ./crates/carol/Cargo.toml; };

          # define an unexported package for the workspace dependencies, which
          # will get reused in the per-crate packages below
          cargoArtifacts = craneLib.buildDepsOnly (with carolCrateMetadata; {
            inherit version src;
            pname = "carol-workspace";
          });

          # helper function for providing a package definition given a
          # Cargo.toml in the workspace
          workspaceCratePackage = crateDir:
            let
              cargoToml = ./. + "/${crateDir}/Cargo.toml";
              cargoMeta = craneLib.crateNameFromCargoToml { inherit cargoToml; };
            in
              craneLib.buildPackage (cargoMeta // {
                inherit src cargoArtifacts crateDir; # share dependencies between the different packages
                doCheck = false; # tests are run in nix flake check with cargo nextest
                cargoExtraArgs = "-p ${cargoMeta.pname}";
              });

          readTomlFile = path: builtins.fromTOML (builtins.readFile path);

          # define packages for the crates listed in the workspace
          cratePackages = builtins.map
            workspaceCratePackage
            ((readTomlFile ./Cargo.toml).workspace.members);

          # partition the packages to keep the flake packages sensible
          packagesToAttrs = list: builtins.listToAttrs (builtins.map (p: {name = p.pname; value = p;}) list);
          isExample = p: (builtins.substring 0 15 p.crateDir) != "example-crates/";
          exportedPackages = builtins.filter isExample cratePackages;
          examplePackages = builtins.filter isExample cratePackages;

          carolToolchain = pkgs.buildEnv {
            name = "carol-toolchain";
            paths = exportedPackages; # TODO only export carol & carlo?
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
