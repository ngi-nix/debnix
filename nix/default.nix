{
  self,
  nixpkgs,
  rust-overlay,
  flake-utils,
  flake-compat, # only here so we don't support `...`
}:
flake-utils.lib.eachSystem (nixpkgs.lib.remove "x86_64-darwin" (nixpkgs.lib.remove "aarch64-darwin" flake-utils.lib.defaultSystems)) (system: let
  name = "poddy";
  root = nixpkgs.lib.cleanSource ./..;
  src = root;
  CARGO_LOCK = "${root}/Cargo.lock";
  CARGO_TOML = "${root}/Cargo.toml";
  RUST_TOOLCHAIN = "${root}/rust-toolchain.toml";
  cargoToml = builtins.path {
    path = "${CARGO_TOML}";
    name = "Cargo.toml";
  };
  cargoLock = builtins.path {
    path = "${CARGO_LOCK}";
    name = "Cargo.lock";
  };
  cargoDeps = pkgs.rustPlatform.importCargoLock {lockFile = cargoLock;};

  overlays = [(import rust-overlay)];

  pkgs = import nixpkgs {
    inherit system overlays;
  };

  rustToolchainToml = pkgs.rust-bin.fromRustupToolchainFile "${RUST_TOOLCHAIN}";
  rustc = rustToolchainToml;
  cargo = rustToolchainToml;
  clippy = rustToolchainToml;
  rustfmt = rustToolchainToml;
  rust-analyzer = rustToolchainToml;

  nativeBuildInputs = [
  ];
  buildInputs = [
  ];
  devInputs = [
    rust-analyzer

    # just a programm runner
    pkgs.just

    #alternative linker
    pkgs.mold
    pkgs.clang
  ];
  fmtInputs = [
    # formatting
    pkgs.alejandra
    pkgs.treefmt
  ];
  editorConfigInputs = [
    pkgs.editorconfig-checker
  ];

in rec {
  # `nix build`
  packages.default =
    (pkgs.makeRustPlatform {
      inherit cargo rustc;
    })
    .buildRustPackage {
      cargoDepsName = name;
      version = "0.1.0";
      inherit src name buildInputs nativeBuildInputs;
      cargoLock.lockFile = cargoLock;
      # checkInputs = [clippy];
      # preCheck = ''
      #   export HOME=$TMPDIR
      # '';
      # postCheck = ''
      #   cargo clippy
      # '';
    };

  # `nix run`
  apps.default = flake-utils.lib.mkApp {
    drv = packages.default;
  };

  devShells = {
    poddy = pkgs.callPackage ./devShell.nix {
      inherit buildInputs pkgs;
      nativeBuildInputs = nativeBuildInputs ++ buildInputs ++ devInputs ++ fmtInputs ++ editorConfigInputs;
    };
    fmtShell = pkgs.mkShell {
      name = "fmt-shell";
      nativeBuildInputs = fmtInputs;
    };
    editorConfigShell = pkgs.mkShell {
      name = "editor-config-shell";
      nativeBuildInputs = editorConfigInputs;
    };
  };
  devShell = devShells.poddy;
  formatter = pkgs.alejandra;
})
