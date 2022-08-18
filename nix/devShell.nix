{
  mkShell,
  clangStdenv,
  pkgs,
  buildInputs,
  nativeBuildInputs,
}:
mkShell {
# clangStdenv.mkDerivation {
  name = "poddy-dev";
  inherit buildInputs nativeBuildInputs;

  ### ENVIRONMENT VARIABLES
  #RUSTFLAGS = "-Z macro-backtrace";
  RUST_BACKTRACE = "full";
  # Development database env variable for `diesel-cli`
  DATABASE_URL = "/tmp/poddy-database.db";
  #########################
  RUSTFLAGS="-C linker=clang -C link-arg=-fuse-ld=${pkgs.mold}/bin/mold -C target-cpu=native";
}
