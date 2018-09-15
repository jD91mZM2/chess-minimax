with import <nixpkgs> {};
stdenv.mkDerivation rec {
  name = "chess-minimax";
  buildInputs = [ gtk3 ];
}
