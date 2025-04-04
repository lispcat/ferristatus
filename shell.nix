{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    pkg-config
    alsa-lib
    alsa-utils
    rustc
    cargo
  ];

  ALSA_PLUGIN_DIR = "${pkgs.alsa-plugins}/lib/alsa-lib";
  LD_LIBRARY_PATH = "${pkgs.alsa-lib}/lib";
  PKG_CONFIG_PATH = "${pkgs.alsa-lib}/lib/pkgconfig";
}
