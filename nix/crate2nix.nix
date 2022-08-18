{
  pkgs,
  crate2nix,
  name,
  src,
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
          # Crate dependency overrides go here
          cairo-sys-rs = attrs: {
            nativeBuildInputs = [pkgs.pkg-config pkgs.cairo];
          };
          gobject-sys = attrs: {
            buildInputs = [pkgs.glib];
            nativeBuildInputs = [pkgs.gob2 pkgs.pkg-config];
          };
          graphene-sys = attrs: {
            buildInputs = [pkgs.graphene pkgs.pkg-config pkgs.glib];
            nativeBuildInputs = [pkgs.graphene pkgs.pkg-config pkgs.glib];
          };
          gio-sys = attrs: {
            buildInputs = [pkgs.glib];
            nativeBuildInputs = [pkgs.pkg-config];
          };
          gdk-pixbuf-sys = attrs: {
            buildInputs = [pkgs.gdk-pixbuf];
            nativeBuildInputs = [pkgs.pkg-config];
          };
          pango-sys = attrs: {
            buildInputs = [pkgs.pango];
            nativeBuildInputs = [pkgs.pkg-config];
          };
          gdk4-sys = attrs: {
            buildInputs = [pkgs.gtk4];
            nativeBuildInputs = [pkgs.pkg-config];
          };
          gsk4-sys = attrs: {
            buildInputs = [pkgs.gtk4];
            nativeBuildInputs = [pkgs.pkg-config];
          };
          gtk4-sys = attrs: {
            buildInputs = [pkgs.gtk4];
            nativeBuildInputs = [pkgs.pkg-config];
          };
          libadwaita-sys = attrs: {
            buildInputs = [pkgs.libadwaita];
            nativeBuildInputs = [pkgs.pkg-config];
          };
          gstreamer-sys = attrs: {
            buildInputs = [pkgs.gst_all_1.gstreamer];
            nativeBuildInputs = [pkgs.pkg-config];
          };
          gstreamer-base-sys = attrs: {
            buildInputs = [pkgs.gst_all_1.gstreamer];
            nativeBuildInputs = [pkgs.pkg-config];
          };
          gstreamer-video-sys = attrs: {
            buildInputs = [
              pkgs.gst_all_1.gstreamer
              pkgs.gst_all_1.gst-plugins-base
              pkgs.gst_all_1.gst-plugins-bad
              pkgs.gst_all_1.gst-plugins-good
            ];
            nativeBuildInputs = [
              pkgs.pkg-config
            ];
          };
          gstreamer-player-sys = attrs: {
            buildInputs = [
              pkgs.gst_all_1.gst-plugins-base
              pkgs.gst_all_1.gst-plugins-bad
              pkgs.gst_all_1.gst-plugins-good
            ];
            nativeBuildInputs = [
              pkgs.pkg-config
            ];
          };
        };
    };
in
  project.rootCrate.build
