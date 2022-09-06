{ self
, nixpkgs
, crate2nix
, rust-overlay
, flake-utils
, flake-compat
, # only here so we don't support `...`
}:
flake-utils.lib.eachSystem (nixpkgs.lib.remove "x86_64-darwin" (nixpkgs.lib.remove "aarch64-darwin" flake-utils.lib.defaultSystems)) (system:
  let
    name = "debnix";
    pname = name;
    root = nixpkgs.lib.cleanSource ./..;
    src = root;
    CARGO_LOCK = "${root}/Cargo.lock";
    CARGO_TOML = "${root}/Cargo.toml";
    RUST_TOOLCHAIN = "${root}/rust-toolchain.toml";
    cargoToml = builtins.path {
      path = "${CARGO_TOML}";
      name = "Cargo.toml";
    };
    cargoLock = {
      lockFile = builtins.path {
        path = src + "/Cargo.lock";
        name = "Cargo.lock";
      };
    };

    cargoDeps = pkgs.rustPlatform.importCargoLock { lockFile = cargoLock; };

    overlays = [ (import rust-overlay) ];

    pkgs = import nixpkgs {
      inherit system overlays;
    };

    crate2nixPkgs = import nixpkgs {
      inherit system;
      overlays = [
        (self: _: {
          rustc = rustToolchainToml;
          cargo = rustToolchainToml;
        })
      ];
    };

    rustToolchainToml = pkgs.rust-bin.fromRustupToolchainFile "${RUST_TOOLCHAIN}";
    rustc = rustToolchainToml;
    cargo = rustToolchainToml;
    clippy = rustToolchainToml;
    rustfmt = rustToolchainToml;
    rust-analyzer = rustToolchainToml;

    nativeBuildInputs = [
      pkgs.pkg-config
      pkgs.installShellFiles
    ];
    buildInputs = [
      pkgs.openssl
    ];
    devInputs = [
      rust-analyzer

      # just a program runner
      pkgs.just
      # for sponge
      pkgs.moreutils

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
    postInstall = ''
      # explicit behavior
      $out/bin/debnix setup --generate-completion bash > ./completions.bash
      installShellCompletion --bash --name ${pname}.bash ./completions.bash
      $out/bin/debnix setup --generate-completion fish > ./completions.fish
      installShellCompletion --fish --name ${pname}.fish ./completions.fish
      $out/bin/debnix setup --generate-completion zsh > ./completions.zsh
      installShellCompletion --zsh --name _${pname} ./completions.zsh
    '';
  in
  rec {
    # `nix build`
    packages.default =
      (pkgs.makeRustPlatform {
        inherit cargo rustc;
      }).buildRustPackage {
        cargoDepsName = name;
        version = "0.1.0";
        inherit src name buildInputs nativeBuildInputs postInstall cargoLock;
        # checkInputs = [clippy];
        # preCheck = ''
        #   export HOME=$TMPDIR
        # '';
        # postCheck = ''
        #   cargo clippy
        # '';
      };
    packages.debnix-crate = crate2nixPkgs.callPackage ./crate2nix.nix {
      inherit
        crate2nix
        name
        nativeBuildInputs
        postInstall
        src
        ;
    };
    packages.control2json =
      (pkgs.makeRustPlatform {
        inherit cargo rustc;
      }).buildRustPackage rec {
        name = "control2json";
        pname = name;
        cargoDepsName = name;
        version = "0.1.0";
        inherit src buildInputs nativeBuildInputs cargoLock;
        buildPhase = ''
          cargo build --package ${name} --release
          mkdir -p $out/bin;
          cp target/release/${name} $out/bin/${name}
        '';
        installPhase = ":";
        checkPhase = ":";
      };

    # `nix run`
    apps.default = flake-utils.lib.mkApp {
      drv = packages.default;
    };
    apps.control2json = flake-utils.lib.mkApp {
      drv = packages.control2json;
    };

    devShells = {
      default = pkgs.callPackage ./devShell.nix {
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
    formatter = pkgs.alejandra;
    nixpkgs = pkgs;
  })
