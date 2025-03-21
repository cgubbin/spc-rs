{
  description = "Flake for Stacked";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      fenix,
      flake-utils,
      advisory-db,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        fenix-pkgs = fenix.packages.${system};

        inherit (pkgs) lib;

        craneLib = (crane.mkLib pkgs).overrideToolchain fenix-pkgs.stable.toolchain;
        # fenix-pkgs.latest.toolchain;
        src = craneLib.cleanCargoSource ./crate;
        cargoToml = craneLib.cleanCargoSource ./crate/Cargo.toml;
        cargoLock = craneLib.cleanCargoSource ./crate/Cargo.lock;

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src cargoToml cargoLock;
          strictDeps = true;

          buildInputs =
            with pkgs;
            [
              curl
              gfortran
              (lib.getLib gfortran.cc)
              openblas
              openssl
              # cpp-netlib
              # Add additional build inputs here
            ]
            ++ lib.optionals stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              libiconv
              darwin.apple_sdk.frameworks.CoreText
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.SystemConfiguration
            ];

          # Additional environment variables can be set directly
          # MY_CUSTOM_VAR = "some value";
        };

        craneLibLLvmTools = craneLib.overrideToolchain (
          fenix.packages.${system}.complete.withComponents [
            "cargo"
            "llvm-tools"
            "rustc"
          ]
        );

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        stacked = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            postUnpack = ''
              cd $sourceRoot/crate
              sourceRoot="."
            '';
          }
        );
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit stacked;

          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          my-crate-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          my-crate-doc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });

          # Check formatting
          my-crate-fmt = craneLib.cargoFmt { inherit src; };

          # Audit dependencies
          my-crate-audit = craneLib.cargoAudit { inherit src advisory-db; };

          # Audit licenses
          # my-crate-deny = craneLib.cargoDeny {
          #   inherit src;
          # };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `my-crate` if you do not want
          # the tests to run twice
          my-crate-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
            }
          );

          my-crate-coverage = craneLib.cargoTarpaulin (commonArgs // { inherit cargoArtifacts; });
          # Ensure that cargo-hakari is up to date

        };

        packages =
          {
            default = stacked;
          }
          // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
            my-crate-llvm-coverage = craneLibLLvmTools.cargoLlvmCov (commonArgs // { inherit cargoArtifacts; });
          };

        apps.default = flake-utils.lib.mkApp { drv = stacked; };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = [
            pkgs.lldb
            pkgs.vscode-extensions.vadimcn.vscode-lldb
            fenix-pkgs.rust-analyzer
            pkgs.cargo-expand
            pkgs.hexyl
          ];

          shellHook = ''
            export RUST_ANALYZER_PATH=${fenix-pkgs.rust-analyzer}/bin/rust-analyzer
            export LLDB_PATH=${pkgs.lldb}/bin/lldb
            export LLDB_DYLIB_PATH=${pkgs.lldb}/lib/liblldb.dylib
          '';
        };
      }
    );
}
