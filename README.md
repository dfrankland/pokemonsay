# `pokemonsay`

Pokemon version of cowsay with CLI and API.

![][1]

Inspired by another [`pokemonsay`][2] and [`parrotsay-api`][3] I created this to
quell my obsession with Pokemon and command line greetings. This module includes
all Pokemon and forms available on [Bulbapedia][4] excluding shinies, _a total
of **846**_.

## CLI

### Random

Simply call `pokemonsay` to get a random `Wild POKEMON appeared!` message.

### Piping to `STDIN`

Pipe to `pokemonsay` to get a random Pokemon with the piped message below it.

## API

There are a few methods that are available to utilize `pokemonsay` in your own
app/module!

### `iChooseYou`

*   Argument: Object `{ number, pokemon, form }`
*   Result: Array of Objects `[{ number, pokemon, form, say }]`

Filter through the database using the available info to get matching Pokemon.
The resulting objects in an array will contain the following properties:

*   `number`: National Dex number (integer).
*   `pokemon`: The name of the Pokemon.
*   `form`: The first letter of the form.
    *   Example: "A" for Alola Form (Marowak A-form) or "A" for Attack Form
        (Deoxys A-form)
*   `say`: The ANSI compatible string meant for `console.log`ging.

### `random`

*   Arguments: Integers `min, max`
*   Result: Object `{ number, pokemon, form, say }`

Get a random `pokemonsay` object. An optional `min` and `max` argument can be
used to get Pokemon from a specific generation (including their alternate forms
regardless of generation). The resulting object will contain the following
properties:

*   `number`: National Dex number (integer).
*   `pokemon`: The name of the Pokemon.
*   `form`: The first letter of the form.
    *   Example: "A" for Alola Form (Marowak A-form) or "A" for Attack Form
        (Deoxys A-form)
*   `say`: The ANSI compatible string meant for `console.log`ging.

### `say`

#### default text (`Wild POKEMON appeared!`)

*   Arguments: Object `{ pokemon, form, options }`
*   Result: String

Returns a string that shows a box with the message `Wild POKEMON appeared!`
with the specified Pokemon and form. The `options` property is an object that
will override the default settings for [`boxen`][5].

#### custom text

*   Arguments: Object `{ text, options }`
*   Result: String

Returns a string that shows a box with the message specified. The `options`
property is an object that will override the default settings for [`boxen`][5].

[1]: https://raw.githubusercontent.com/dfrankland/pokemonsay/master/demo.png
[2]: https://github.com/possatti/pokemonsay
[3]: https://github.com/matheuss/parrotsay-api
[4]: http://bulbapedia.bulbagarden.net/wiki/List_of_Pok%C3%A9mon_by_National_Pok%C3%A9dex_number
[5]: https://github.com/sindresorhus/boxen
