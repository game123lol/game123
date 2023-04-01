let
  nixpkgs = import <nixpkgs> {};
in

  with nixpkgs;

  mkShell {
    buildInputs = [
      alsaLib
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
    ];
    
    LD_LIBRARY_PATH = lib.makeLibraryPath [
      wayland
      egl-wayland
    ];
  }