{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    # helper utils to write nice flakes
    flake-parts.url = "github:hercules-ci/flake-parts";
    # rust build helper
    naersk.url = "github:nix-community/naersk";
    # esp-idf packaged for nix with c compilers
    nixpkgs-esp-dev.url = "github:madmo/nixpkgs-esp-dev/use-patchelf-instead-of-fhsenv";
    esp32 = {
      url = "github:madmo/esp32-idf-rust";
      #inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ self, nixpkgs, flake-parts, naersk, nixpkgs-esp-dev, esp32, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      perSystem = { pkgs, system, ... }:
        let
          toolchain = esp32.packages.x86_64-linux.toolchain;

          naersk' = pkgs.callPackage naersk {
            cargo = toolchain;
            rustc = toolchain;
          };

          esp-idf = (pkgs.esp-idf-full.override {
            rev = "v5.3.3";
            sha256 = "sha256-ESX+K0BrIKEYKzgdRlwZqAY7EvrQZ88V9xr7PtBofI4=";
          });
        in {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import "${nixpkgs-esp-dev}/overlay.nix") ];
          };

          packages.default = naersk'.buildPackage {
            src = ./.;

            additionalCargoLock =
              "${toolchain}/.rustup/toolchains/esp/lib/rustlib/src/rust/library/Cargo.lock";
            copyBins = false;
            copyTarget = true;
            singleStep = true;

            nativeBuildInputs = [
              esp-idf
              pkgs.ldproxy
              esp32.packages.x86_64-linux.toolchain-hook
            ];
          };

          devShells.default = pkgs.mkShell {
            packages = [ pkgs.bacon pkgs.espflash pkgs.python3 esp-idf pkgs.ldproxy toolchain esp32.packages.x86_64-linux.toolchain-hook ];
          };
        };
      flake = { };
    };
}
