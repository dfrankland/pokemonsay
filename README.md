# `pokemonsay`

Pokemon version of cowsay, powered by [PokeAPI][1].

https://github.com/user-attachments/assets/36cc94e7-5bec-4235-88eb-1cf190d562b4

Inspired by another [`pokemonsay`][2] and [`parrotsay-api`][3] I created this to
quell my obsession with Pokemon and command line greetings.

## CLI

Simply call `pokemonsay` to get a random `Wild POKEMON appeared!` message.

Pipe to `pokemonsay` to get a random Pokemon with the piped message below it.

### Install

Install with Cargo:

```bash
cargo install pokemonsay
```

Or use the Nix flake:

```bash
nix run 'github:dfrankland/pokemonsay'
```

### Usage

Run `pokemonsay --help` to see all available options.

#### Basic Usage

Display a random Pokemon with the default message:

```bash
pokemonsay
```

Pipe a custom message to display with a random Pokemon:

```bash
echo "Your message here" | pokemonsay
```

#### Configuration Options

**`--query-method <METHOD>`**

- Specifies where to fetch Pokemon data from
- Options: `db`, `http`
- Default: `db` (if embedded), `http` (otherwise)
- `db`: Queries a local SQLite database (faster, no rate limit)
- `http`: Queries PokeAPI GraphQL endpoint (rate limit: 200 queries/hour)

**`--db-path <PATH>`**

- Path to the PokeAPI SQLite database
- Can also be set via `POKEMONSAY_DB_PATH` environment variable
- Required when using `--query-method db` (in non-embedded builds)
- Example: `pokemonsay --db-path ./pokeapi.db`

**`--sprites-retrieval-method <METHOD>`**

- Specifies where to fetch Pokemon sprite images from
- Options: `embedded`, `http`
- Default: `embedded` (if available), `http` (otherwise)
- `embedded`: Uses sprites built into the CLI (fastest)
- `http`: Downloads from PokeAPI sprites repository

**`--pokemonsay-template <TEMPLATE>`**

- Template for the message displayed below the Pokemon sprite
- Uses TinyTemplate syntax with `{pokemon}` as the Pokemon name placeholder
- Default: `"Wild {pokemon} appeared!"`
- Example: `pokemonsay --pokemonsay-template "I choose you, {pokemon}!"`

**`--crop-sprite-transparent-bg`**

- Flag to crop transparent pixels from the Pokemon sprite background
- When enabled, removes padding around the sprite for a tighter display
- Example: `pokemonsay --crop-sprite-transparent-bg`

**`--max-sprite-dimension <DIMENSION>`**

- Maximum width or height for displaying the Pokemon sprite, preserving aspect
  ratio
- Set to `0` to disable scaling (useful for terminals without image support)
- Default: `30` (for terminals that support Kitty, iTerm2, or Sixel graphics
  protocols), `0` (for others)
- Example: `pokemonsay --max-sprite-dimension 50`

#### Advanced Database Options

These options are used in conjunction with the SQLite database from PokeAPI.

**`--db-pokemon-query <QUERY>`**

- Custom SQL query to fetch Pokemon data (when using `--query-method db`)
- Use `pokemonsay --help` to see the default query

**`--db-sprites-query <QUERY>`**

- Custom SQL query to fetch Pokemon sprite URLs (when using `--query-method db`)
- Use `pokemonsay --help` to see the default query

**`--db-species-name-query <QUERY>`**

- Custom SQL query to fetch Pokemon species names (when using `--query-method db`)
- Use `pokemonsay --help` to see the default query

#### Advanced HTTP Options

This option is used in conjunction with the PokeAPI GraphQL endpoint which you
can play with here: <https://graphql.pokeapi.co/v1beta2/console/>

**`--http-graphql-query <QUERY>`**

- Custom GraphQL query to fetch Pokemon data (when using `--query-method http`)
- Use `pokemonsay --help` to see the default query

#### Examples

Display with HTTP query method and custom template:

```bash
pokemonsay --query-method http --pokemonsay-template "{pokemon} is awesome!"
```

Display with transparent background cropping and custom sprite size:

```bash
pokemonsay --crop-sprite-transparent-bg --max-sprite-dimension 40
```

Use a custom database:

```bash
pokemonsay --query-method db --db-path /path/to/pokeapi.db
```

Pipe a message with custom styling:

```bash
echo "🎮 Get ready for battle! 🎮" | pokemonsay --crop-sprite-transparent-bg
```

## Building

This crate is built with a SQLite database from [PokeAPI][4]. The database and
sprite assets can be included in the binary at compile-time or referenced at
runtime, depending on your build configuration and needs.

### Using Nix (Recommended)

The recommended way to build and package `pokemonsay` is with Nix, which
automates the PokeAPI database and sprites extraction:

```bash
nix build
# outputs the binary to `./result/bin/pokemonsay`
```

This approach:

- Automatically fetches and builds the PokeAPI database and sprites
- Handles all dependencies reproducibly
- Produces a self-contained binary with embedded assets (by default)

### Manual Build (Without Nix)

If you're not using Nix, follow these steps:

1. **Clone and build the PokeAPI database:**

   Clone the [PokeAPI repository][4] and follow its build instructions:

   ```bash
   git clone --recurse-submodules https://github.com/PokeAPI/pokeapi.git
   cd ./pokeapi
   make setup
   make build-db
   ```

2. **Build `pokemonsay`:**

   ```bash
   EMBED_DB_PATH="$(realpath ./pokeapi/db.sqlite3)" \
   EMBED_SPRITES_PATH="$(realpath ./pokeapi/data/v2/sprites/sprites)" \
   cargo build --release --features embed-db,embed-sprites
   ```

### Build Features

`pokemonsay` provides two compile-time features that control what assets are
embedded in the binary:

#### `embed-db` (default)

- **What it does:** Embeds the PokeAPI SQLite database directly into the
  compiled binary
- **Default:** Enabled
- **Trade-offs:**
  - ✅ Queries are faster (no disk I/O)
  - ✅ No need to distribute a separate database file
  - ✅ No rate limiting from PokeAPI (independent of HTTP queries)
  - ❌ Increases binary size

When enabled, you can still use `--query-method http` to query PokeAPI instead,
or `--db-path` to specify an external database.

#### `embed-sprites` (optional)

- **What it does:** Embeds all Pokemon sprite images into the binary
- **Default:** Disabled
- **Trade-offs:**
  - ✅ Sprite retrieval is instant (no network requests)
  - ✅ Works offline
  - ✅ `--sprites-retrieval-method embedded` is available
  - ❌ Increases binary size

When enabled, you can still use `--sprites-retrieval-method http` to download
sprites instead.

### Recommended Build Configurations

**Fastest, largest binary (full embedding):**

```bash
cargo build --release --features embed-db,embed-sprites
```

**Balanced (embedded database, HTTP sprites):**

```bash
cargo build --release --features embed-db
```

**Smallest binary (requires runtime setup):**

```bash
cargo build --release --no-default-features
```

Defaults to using HTTP calls to PokeAPI.

Optionally, provide database and sprites at runtime:

```bash
pokemonsay --query-method db --db-path /path/to/pokeapi.db
```

## Previous version

Looking for the previous version written in JavaScript? Find it on [the `v1`
branch][5].

[1]: https://pokeapi.co/
[2]: https://github.com/possatti/pokemonsay
[3]: https://github.com/matheuss/parrotsay-api
[4]: https://github.com/PokeAPI/pokeapi
[5]: https://github.com/dfrankland/pokemonsay/tree/v1
