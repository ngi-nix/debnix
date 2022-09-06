{
  inputs = {
    # nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nixpkgs.url = "github:a-kenji/nixpkgs/mold";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat.url = "github:edolstra/flake-compat";
    flake-compat.flake = false;
    crate2nix.url = "github:kolloch/crate2nix";
    crate2nix.flake = false;
  };

  outputs = {...} @ args: let
    cargoLock = builtins.path {
      path = ./Cargo.lock;
      name = "Cargo.lock";
    };
  in
    import ./nix args;
}
