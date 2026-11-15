# Builds PokeAPI's SQLite database from its CSV data, with the Python
# dependencies pinned by PokeAPI's own uv.lock (read by uv2nix).
{
  lib,
  stdenvNoCC,
  fetchFromGitHub,
  callPackage,
  python3,
  # The `pokeapi` flake input. It is fetched at evaluation time, so reading its
  # uv.lock below is not import-from-derivation.
  pokeapiSrc,
  pyproject-nix,
  uv2nix,
  pyproject-build-systems,
}: let
  # GitHub tarballs leave git submodules as empty directories, so the sprites
  # submodule is fetched on its own. `rev` must be the submodule commit that
  # the pinned PokeAPI rev points at; run `update-pokeapi` in the devshell to
  # bump both together. (PokeAPI's other submodule, cries, is not needed.)
  sprites = fetchFromGitHub {
    owner = "PokeAPI";
    repo = "sprites";
    rev = "bf4c47ac82c33b330e33d98b8882d1cedb2f53e7";
    hash = "sha256-wFAeI/xSyIoIcSIu7prP+UallOUdjSbKlQdoz80Od+Y=";
  };

  workspace = uv2nix.lib.workspace.loadWorkspace {workspaceRoot = pokeapiSrc.outPath;};

  pythonSet =
    (callPackage pyproject-nix.build.packages {python = python3;}).overrideScope
    (lib.composeManyExtensions [
      pyproject-build-systems.overlays.wheel
      (workspace.mkPyprojectOverlay {sourcePreference = "wheel";})
    ]);

  # `deps.default` is `[project.dependencies]`, without the dev group.
  venv = pythonSet.mkVirtualEnv "pokeapi-env" workspace.deps.default;
in
  stdenvNoCC.mkDerivation {
    pname = "pokeapi";
    version = pokeapiSrc.rev;
    src = pokeapiSrc;

    nativeBuildInputs = [venv];

    # PokeAPI's Makefile targets shell out to `uv run`, which would resolve and
    # download packages itself, so this calls manage.py directly. It mirrors
    # `make setup` and `make build-db`.
    buildPhase = ''
      runHook preBuild

      rmdir data/v2/sprites
      ln -s ${sprites} data/v2/sprites

      # build_all() decides which sprite paths exist from a manifest it
      # downloads from the *latest* PokeAPI/sprites release, falling back to
      # the pinned submodule only when offline. The Linux sandbox has no
      # network, but the Darwin one does, so without this the database would
      # depend on the build platform and the day it was built.
      export POKEAPI_USE_LOCAL_SPRITES=1

      python manage.py migrate --settings=config.local
      echo "from data.v2.build import build_all; build_all()" \
        | python manage.py shell --settings=config.local

      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall
      mkdir -p $out
      cp db.sqlite3 $out/
      # A symlink rather than a copy keeps ~1.7 GB of sprites from being
      # duplicated in the store (and the binary cache).
      ln -s ${sprites}/sprites $out/sprites
      runHook postInstall
    '';
  }
