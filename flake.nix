{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
  };

  outputs = {
    self,
    fenix,
    flake-utils,
    naersk,
    nixpkgs,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      name = "sudoku";
      pkgs = nixpkgs.legacyPackages.${system};

      toolchain = with fenix.packages.${system};
        combine [
          stable.cargo
          stable.rustc
          stable.rust-std
          stable.rust-src
          stable.rust-analyzer
        ];

      nativeBuildInputs = with pkgs; [
        toolchain
        pkg-config
      ];

      buildInputs = with pkgs; [
        glibc
        alsa-lib
        dbus
        fontconfig
        libudev-zero
        mold
        udev
        vulkan-tools
        vulkan-headers
        vulkan-loader
        vulkan-validation-layers
        libxkbcommon
        wayland
        xorg.libX11
        xorg.libXrandr
        xorg.libXcursor
        xorg.libXi
        clang
      ];
      shellHook = ''
        export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${nixpkgs.lib.makeLibraryPath buildInputs}"
      '';

      naersk' = pkgs.callPackage naersk {
        cargo = toolchain;
        rustc = toolchain;
      };
    in {
      defaultPackage = naersk'.buildPackage {
        name = name;
        src = ./.;
        inherit nativeBuildInputs buildInputs shellHook;
      };

      devShell = pkgs.mkShell {
        name = name;
        inherit nativeBuildInputs buildInputs shellHook;
      };
    });
}
