{
  description = "Description for the project";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    pyproject-nix = {
      url = "github:pyproject-nix/pyproject.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    uv2nix = {
      url = "github:pyproject-nix/uv2nix";
      inputs.pyproject-nix.follows = "pyproject-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pyproject-build-systems = {
      url = "github:pyproject-nix/build-system-pkgs";
      inputs.pyproject-nix.follows = "pyproject-nix";
      inputs.uv2nix.follows = "uv2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # The rev is pinned so that a bare `nix flake update` cannot move PokeAPI
    # without also moving the sprites fetched in pokeapi.nix. Bump both with
    # `update-pokeapi` from the devshell.
    pokeapi = {
      url = "github:PokeAPI/pokeapi/5cbeb3f3eb55a9b86da52c4869336f9d61fdd6d7";
      flake = false;
    };
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    devshell.url = "github:numtide/devshell";
  };

  outputs = inputs @ {
    flake-parts,
    crane,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        flake-parts.flakeModules.easyOverlay
      ];
      # No x86_64-darwin: nixpkgs dropped it in 26.11.
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin"];
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
            (inputs.rust-overlay.lib.mkRustBin {} p).selectLatestNightlyWith (toolchain:
              toolchain.default.override {
                extensions = ["rust-src"];
              })
        );
        pokeapi = pkgs.callPackage ./pokeapi.nix {
          pokeapiSrc = inputs.pokeapi;
          inherit (inputs) pyproject-nix uv2nix pyproject-build-systems;
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
            version = "2.0.4";
            sha256 = "sha256-jQda7XP8w8SCqlmCcyf3N3apaMHBrbdVgMTCmqBa3c4=";
          };
          strictDeps = true;
        };
        pokemonsay = craneLib.buildPackage {
          pname = "pokemonsay";
          meta.mainProgram = "pokemonsay";
          src = craneLib.cleanCargoSource ./.;
          nativeBuildInputs = [pkgs.pkg-config];
          buildInputs = [pkgs.openssl];
          cargoExtraArgs = "--features embed-db,embed-sprites";
          EMBED_DB_PATH = "${pokeapi-optimized}/db.sqlite3";
          EMBED_SPRITES_PATH = "${pokeapi-optimized}/sprites";
        };
      in {
        packages = {
          default = pokemonsay;
          inherit pokeapi pokeapi-full pokeapi-optimized sea-orm-cli pokemonsay;
        };

        overlayAttrs = {
          inherit pokemonsay;
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
