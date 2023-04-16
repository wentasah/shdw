{
  description = "shdw – Tool to manage symlinks to a shadow directory";

  # Nixpkgs / NixOS version to use.
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }: {
    overlays.default = final: prev: {
      shdw = with final;
        rustPlatform.buildRustPackage rec {
          pname = "shdw";
          version = "0.1.0";
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [ installShellFiles ];
          nativeCheckInputs = [
            (bats.withLibraries (p: [ p.bats-support p.bats-assert p.bats-file ]))
          ];

          postCheck = ''
            PATH=$PWD/$(echo target/*/release):$PATH
            bats -F pretty tests
          '';

          preFixup = ''
                dir=($releaseDir/build/shdw-*/out)
                installShellCompletion $dir/shdw.{bash,fish} --zsh $dir/_shdw
                installManPage $dir/shdw*.1
              '';
        };
    };
  } // flake-utils.lib.eachDefaultSystem (system:
    let pkgs = import nixpkgs { inherit system; overlays = [ self.overlays.default ]; };
    in {
      packages = rec {
        shdw = pkgs.shdw;
        default = shdw;
      };
      apps = rec {
        shdw = flake-utils.lib.mkApp { drv = self.packages.${system}.shdw; };
        default = shdw;
      };
    }
  );
}
