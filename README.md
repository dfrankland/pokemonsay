# `pokemonsay`

Pokemon version of cowsay, powered by [PokeAPI][1].

https://github.com/user-attachments/assets/36cc94e7-5bec-4235-88eb-1cf190d562b4

Inspired by another [`pokemonsay`][2] and [`parrotsay-api`][3] I created this to
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

## Previous version

Looking for the previous version written in JavaScript? Find it on [the `v1`
branch][4].

[1]: https://pokeapi.co/
[2]: https://github.com/possatti/pokemonsay
[3]: https://github.com/matheuss/parrotsay-api
[4]: https://github.com/dfrankland/pokemonsay/tree/v1
