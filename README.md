# `pokemonsay`

Pokemon version of cowsay, powered by [PokeAPI][1].

![][2]

Inspired by another [`pokemonsay`][3] and [`parrotsay-api`][4] I created this to
quell my obsession with Pokemon and command line greetings.

## CLI

Simply call `pokemonsay` to get a random `Wild POKEMON appeared!` message.

Pipe to `pokemonsay` to get a random Pokemon with the piped message below it.

## Install

Install with Cargo:

```bash
cargo install pokemonsay
```

Or use the Nix flake:

```bash
nix run 'github:dfrankland/pokemonsay'
```

[1]: https://pokeapi.co/
[2]: https://raw.githubusercontent.com/dfrankland/pokemonsay/master/demo.mp4
[3]: https://github.com/possatti/pokemonsay
[4]: https://github.com/matheuss/parrotsay-api
