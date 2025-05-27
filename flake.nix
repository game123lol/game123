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
        cargoLock.outputHashes = {
         "macroquad-0.4.11" = "sha256-fGc8SCvsqMzTF3kt3fHg2DVGnDJLTkk06/kTA1m6nOo=";
        };
        buildInputs = with pkgs; [
          SDL2
          cmake
          alsa-lib
        ];
        nativeBuildInputs = with pkgs; [
          SDL2
          cmake
          alsa-lib
          pkg-config
        ];
        postInstall = ''
          cp -r assets $out/assets
        '';
      };
  	in {
    	defaultPackage = myRustBuild;
    	devShell = pkgs.mkShell {
        LD_LIBRARY_PATH = builtins.concatStringsSep ":" [
          "${pkgs.xorg.libX11}/lib"
          "${pkgs.xorg.libXi}/lib"
          "${pkgs.libGL}/lib"
          "${pkgs.wayland}/lib"
          "${pkgs.libxkbcommon}/lib"
        ];
      	buildInputs = with pkgs; [ 
          (rustVersion.override { extensions = [ "rust-src" "rust-analyzer" ]; }) 
          gdb
          linuxPackages.perf
          cargo-flamegraph
          alsa-lib
          grafx2
          cmake
          freetype
          expat
          openssl
          pkg-config
          fontconfig
          vulkan-validation-layers
          rust-analyzer
          SDL2
          cmake
          alsa-lib
          wayland
          egl-wayland
        ];
    	};
  	});
}
