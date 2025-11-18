# pokeapi_db

Generated from the root directory with:

```bash
nix build '.#pokeapi-optimized'
nix run '.#sea-orm-cli' -- generate entity \
  --database-url sqlite://./result/db.sqlite3 \
  --output-dir ./src/pokeapi_db/ \
  --with-prelude all-allow-unused-imports
```
