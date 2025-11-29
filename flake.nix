{
  description = "Description for the project";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    dream2nix.url = "github:nix-community/dream2nix";
    dream2nix.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    devshell.url = "github:numtide/devshell";
  };

  outputs = inputs @ {
    flake-parts,
    dream2nix,
    crane,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        # To import an internal flake module: ./other.nix
        # To import an external flake module:
        #   1. Add foo to inputs
        #   2. Add foo as a parameter to the outputs function
        #   3. Add here: foo.flakeModule
      ];
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin"];
      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: let
        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain (
          p:
          # NB: use nightly for https://github.com/rust-lang/rustfmt/issues/6241
            p.rust-bin.selectLatestNightlyWith (toolchain:
              toolchain.default.override {
                extensions = ["rust-src"];
              })
        );
      in {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            inputs.rust-overlay.overlays.default
          ];
        };

        packages = let
          pokeapi = dream2nix.lib.evalModules {
            packageSets.nixpkgs = pkgs;
            modules = [
              ./pokeapi.nix
              {
                paths.projectRoot = ./.;
                paths.projectRootFile = "flake.nix";
                paths.package = ./.;
              }
            ];
          };
          pokeapi-full = pkgs.stdenv.mkDerivation {
            name = "pokeapi-full";
            phases = ["buildPhase"];
            buildPhase = ''
              mkdir -p $out/sprites
              cp ${pokeapi}/db.sqlite3 $out/
              cp ${pokeapi}/sprites/pokemon/*.png $out/sprites
            '';
          };
          pokeapi-optimized = pkgs.stdenv.mkDerivation {
            name = "pokeapi-optimized";
            phases = ["buildPhase"];
            buildInputs = [pkgs.sqlite];
            buildPhase = ''
              mkdir -p $out/sprites
              sqlite3 ${pokeapi}/db.sqlite3 ".dump pokemon_v2_pokemon pokemon_v2_pokemonsprites pokemon_v2_pokemonspecies pokemon_v2_pokemonspeciesname pokemon_v2_language" | sqlite3 $out/db.sqlite3
              sqlite3 $out/db.sqlite3 < ${./pokeapi-optimize-db-table.sql} > $out/optimize.sql
              sqlite3 $out/db.sqlite3 < $out/optimize.sql
              rm $out/optimize.sql
              cp -r ${pokeapi}/sprites/pokemon/*.png $out/sprites
            '';
          };
          sea-orm-cli = craneLib.buildPackage {
            src = pkgs.fetchCrate {
              pname = "sea-orm-cli";
              version = "2.0.0-rc.18";
              sha256 = "sha256-5tSvomUB6p1XbQUFJp3+DNzIWAtXgpUDKMoUNQZZ7Ng=";
            };
            strictDeps = true;
          };
          pokemonsay = craneLib.buildPackage {
            pname = "pokemonsay";
            meta.mainProgram = "pokemonsay";
            src = craneLib.cleanCargoSource ./.;
            cargoExtraArgs = "--features embed-db,embed-sprites";
            EMBED_DB_PATH = "${pokeapi-optimized}/db.sqlite3";
            EMBED_SPRITES_PATH = "${pokeapi-optimized}/sprites";
          };
        in {
          default = pokemonsay;
          inherit pokeapi pokeapi-full pokeapi-optimized sea-orm-cli pokemonsay;
        };
        devShells.default = let
          devshell = import "${inputs.devshell}/default.nix" {nixpkgs = pkgs;};
          devshellDevShell = craneLib.devShell.override {
            mkShell = {
              inputsFrom,
              packages,
            }:
              devshell.mkShell {
                imports = [(devshell.importTOML ./devshell.toml)];
                packagesFrom = inputsFrom;
                inherit packages;
              };
          };
        in
          devshellDevShell {
            checks = inputs.self.checks.${system};
            packages = [];
          };
      };
      flake = {
        # The usual flake attributes can be defined here, including system-
        # agnostic ones like nixosModule and system-enumerating ones, although
        # those are more easily expressed in perSystem.
      };
    };
}
