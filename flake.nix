{
  description = "A full Rust flake";

  inputs = {
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
    };
    treefmt-nix.url = "github:numtide/treefmt-nix";
    pre-commit-hooks.url = "github:cachix/git-hooks.nix";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    systems.url = "github:nix-systems/default-linux";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      treefmt-nix,
      flake-parts,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      perSystem =
        {
          config,
          system,
          pkgs,
          ...
        }:
        let
          treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
          rust-bin = pkgs.rust-bin.stable.latest.default.override {
            targets = [ "thumbv7em-none-eabihf" ];
          };
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              inputs.rust-overlay.overlays.default
            ];
          };
          checks = {
            pre-commit-check = inputs.pre-commit-hooks.lib.${system}.run {
              src = ./.;
              # hooks = {
              #   clippy.enable = true;
              #   cargo-check.enable = true;
              # };
            };
            formatting = treefmtEval.config.build.check self;
          };

          devShells.default = pkgs.mkShell {
            nativeBuildInputs =
              with pkgs;
              [
                binutils-unwrapped-all-targets
                probe-rs
                minicom
                gdb

                libgudev.dev

                go-task
              ]
              ++ [ rust-bin ];
          };

          packages.default = let
            rustPlatform = pkgs.makeRustPlatform {
                cargo = rust-bin;
                rustc = rust-bin;
              };
            in
            rustPlatform.buildRustPackage {
            pname = "fantastic-disco";
            version = "0.1.0";

            target = "thumbv7em-none-eabihf";

            buildInputs = with pkgs; [
              binutils-unwrapped-all-targets
            ];

            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            postBuild = ''
              objcopy -O ihex ./target/thumbv7em-none-eabihf/release/fantastic-disco fantastic-disco.hex
            '';

            installPhase = ''
              mkdir -p $out/bin
              cp fantastic-disco.hex $out/bin
            '';
          };

          formatter = treefmtEval.config.build.wrapper;
        };
    };
}
