{
  pkgs,
  crate2nix,
  name,
  src,
  nativeBuildInputs,
  postInstall,
}: let
  #crate2nix
  inherit
    (import "${crate2nix}/tools.nix" {inherit pkgs;})
    generatedCargoNix
    ;

  project =
    import
    (generatedCargoNix {
      inherit src name;
    })
    {
      inherit pkgs;
      defaultCrateOverrides =
        pkgs.defaultCrateOverrides
        // {
          _ = attrs: {
            inherit postInstall;
          };
          # Crate dependency overrides go here
          # gstreamer-player-sys = attrs: {
          #   buildInputs = [
          #     pkgs.gst_all_1.gst-plugins-base
          #     pkgs.gst_all_1.gst-plugins-bad
          #     pkgs.gst_all_1.gst-plugins-good
          #   ];
          #   nativeBuildInputs = [
          #     pkgs.pkg-config
          #   ];
          # };
        };
    };
  build =
    project.rootCrate.build.override {
    };
in
  build
