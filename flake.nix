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
         "egui-tetra-0.3.0" = "sha256-jbV1u97pO+2Ama1VKtyXmB2FtDmEmldYgNi5TPVmvmc=";
        };
        buildInputs = with pkgs; [
          SDL2
          cmake
          alsaLib
        ];
        nativeBuildInputs = with pkgs; [
          SDL2
          cmake
          alsaLib
          pkg-config
        ];
        postInstall = ''
          cp -r assets $out/assets
        '';
      };
  	in {
    	defaultPackage = myRustBuild;
    	devShell = pkgs.mkShell {
      	buildInputs = with pkgs; [ 
          (rustVersion.override { extensions = [ "rust-src" "rust-analyzer" ]; }) 
          gdb
          linuxPackages.perf
          cargo-flamegraph
          alsaLib
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
          alsaLib
          wayland
          egl-wayland
        ];
    	};
  	});
}
