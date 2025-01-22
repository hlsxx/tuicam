{
  description = "Tuicam - Terminal-based camera with switchable modes";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
      };
    in {
      packages = {
        tuicam = pkgs.rustPlatform.buildRustPackage {
          pname = "tuicam";
          version = "0.0.1";
          src = ./.;
          cargoHash = "sha256-wGXRFCirIZ6tWYK73CZJD5Pzx1Nu5iooeZ+Esijfgtw=";

          nativeBuildInputs = with pkgs; [
            pkg-config
            llvmPackages.llvm
            clang
          ];

          buildInputs = with pkgs; [
            opencv
            udev
            llvmPackages.libclang
          ];

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = ''-I${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.llvmPackages.libclang.version}/include'';
          RUSTFLAGS = ''--cfg=libclang_path="${pkgs.llvmPackages.libclang.lib}/lib"'';

          preBuildPhases = ["setEnvVars"];
          setEnvVars = ''
            export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
            export BINDGEN_EXTRA_CLANG_ARGS="-I${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.llvmPackages.libclang.version}/include"
          '';

          meta = with pkgs.lib; {
            description = "Terminal-based camera with switchable modes";
            license = licenses.mit;
            maintainers = with maintainers; [hlsxx];
            platforms = platforms.linux;
          };
        };
      };

      apps = {
        default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.tuicam;
          program = "tuicam";
        };
      };
    });
}
