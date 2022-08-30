{
  mkShell,
  clangStdenv,
  pkgs,
  buildInputs,
  nativeBuildInputs,
}:
mkShell {
  # clangStdenv.mkDerivation {
  name = "debnix";
  inherit buildInputs nativeBuildInputs;

  ### ENVIRONMENT VARIABLES
  #RUSTFLAGS = "-Z macro-backtrace";
  RUST_BACKTRACE = "full";
  #########################
  RUSTFLAGS = "-C linker=clang -C link-arg=-fuse-ld=${pkgs.mold}/bin/mold -C target-cpu=native";
}
