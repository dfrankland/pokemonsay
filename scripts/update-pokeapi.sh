#!/usr/bin/env bash
# Bump PokeAPI to a new commit (default: the latest on master), together with
# the sprites submodule commit that PokeAPI pins at that commit.
#
# The two have to move together. The `pokeapi` flake input is a GitHub tarball,
# which has no submodules, so pokeapi.nix fetches the sprites separately. The
# input's rev is pinned in flake.nix so `nix flake update` cannot bump PokeAPI
# on its own.
#
# Usage: update-pokeapi [<pokeapi commit>]
# Set GITHUB_TOKEN to avoid the anonymous GitHub API rate limit.
set -euo pipefail

cd "${PRJ_ROOT:-$(git rev-parse --show-toplevel)}"

github_api() {
  curl --fail --silent --show-error --location \
    ${GITHUB_TOKEN:+--header "Authorization: Bearer $GITHUB_TOKEN"} \
    "https://api.github.com/repos/PokeAPI/$1"
}

rev=${1:-$(github_api pokeapi/commits/master | jq -r .sha)}
sprites_rev=$(github_api "pokeapi/contents/data/v2/sprites?ref=$rev" | jq -r .sha)

echo "pokeapi: $rev"
echo "sprites: $sprites_rev"
echo "Prefetching sprites (about 2 GB, this takes a while)..."
sprites_hash=$(nix flake prefetch --json "github:PokeAPI/sprites/$sprites_rev" | jq -r .hash)

sed -i -E "s|github:PokeAPI/pokeapi/[0-9a-f]{40}|github:PokeAPI/pokeapi/$rev|" flake.nix
sed -i -E "/repo = \"sprites\";/,/hash = / {
  s|rev = \"[0-9a-f]{40}\"|rev = \"$sprites_rev\"|
  s|hash = \"[^\"]*\"|hash = \"$sprites_hash\"|
}" pokeapi.nix

nix flake lock

echo
echo "Done. Next: nix build '.#pokeapi-optimized', then regenerate the entities"
echo "as described in src/pokeapi_db/README.md if the schema changed."
