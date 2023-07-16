{
inputs = {
	flake-utils.url = "github:numtide/flake-utils";
	rust-overlay.url = "github:oxalica/rust-overlay";
  };
outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
	flake-utils.lib.eachDefaultSystem (system:
  	let
    	overlays = [ (import rust-overlay) ];
    	pkgs = import nixpkgs { inherit system overlays; };
    	rustVersion = pkgs.rust-bin.stable.latest.default;
    	rustPlatform = pkgs.makeRustPlatform {
      	cargo = rustVersion;
      	rustc = rustVersion;
    	};
    	myRustBuild = rustPlatform.buildRustPackage {
      	pname =
        	"game123"; # make this what ever your cargo.toml package.name is
      	version = "0.1.0";
      	src = ./.; # the folder with the cargo.toml
      	cargoLock.lockFile = ./Cargo.lock;
    	};
  	in {
    	defaultPackage = myRustBuild;
    	devShell = pkgs.mkShell {
      	buildInputs = with pkgs; [ 
          (rustVersion.override { extensions = [ "rust-src" "rust-analyzer" ]; }) 
          alsaLib
          grafx2
          cmake
          freetype
          rustup
          cargo
          expat
          openssl
          pkgconfig
          fontconfig
          vulkan-validation-layers
          rust-analyzer
          SDL2
          wayland
          egl-wayland
        ];
    	};
  	});
}