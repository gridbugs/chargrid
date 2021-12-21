with import ./common.nix;
mkDerivation {

  # To just use the current nightly rust:
  # rust = nightly;
  #
  # To use a specific version of nighty rust:
  # rust = rustChannelOf { date = "2021-10-09"; channel = "nightly"; };
  rust = nightly;
}
