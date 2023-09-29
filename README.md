# Just a small roguelike game
To run on NixOS:
```
nix run github:Nuxssss/game123
```
To run on non-NixOS system using nix:
```
nix run --override-input nixpkgs nixpkgs/nixos-23.05 --impure github:guibou/nixGL -- \
  nix run github:Nuxssss/game123
```
To build manually:
```
git clone https://github.com/Nuxssss/game123
cd game123
cargo run --release
```
