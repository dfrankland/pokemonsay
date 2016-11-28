import boxen from 'boxen';
import pokemon from './pokemon.json';

const numberOfPokemon = pokemon.length;

export default {
  iChooseYou: filters => {
    let chosenPokemon = pokemon;
    Object.keys(filters).map(
      filter => {
        chosenPokemon = chosenPokemon
          .filter(
            pkmn => `${pkmn[filter]}`.toLowerCase() === `${filters[filter]}`.toLowerCase()
          );
      }
    );
    return chosenPokemon;
  },

  random: (min = 0, max = numberOfPokemon) => {
    if (min < 0) throw new Error(`Minimum range is 0`);
    if (max > numberOfPokemon) throw new Error(`Max range is ${numberOfPokemon}`);
    const random = Math.floor(Math.random() * (max - min)) + min;
    return pokemon[random];
  },

  say: ({ text, options, pokemon, form }) => {
    if (text) {
      let fixedText = text;

      // Boxen doesn't handle tabs very well...
      fixedText = fixedText.replace(/\t/g, '    ');

      // Also, remove trailing newlines.
      fixedText = fixedText.replace(/\n$/, '');

      // Fully, fill box width of 80 columns
      fixedText = fixedText.split('\n').map(
        line => {
          let rightPaddedLine = line;
          while (rightPaddedLine.length < 80) {
            rightPaddedLine += ' ';
          }

          return rightPaddedLine;
        }
      ).join('\n');

      return boxen(
        fixedText,
        {
          borderStyle: 'double',
          padding: {
            top: 1,
            right: 1,
            bottom: 1,
            left: 1,
          },
          ...options,
        },

      );
    }

    const defaultText = `Wild ${pokemon.toUpperCase()}${form ? ` ${form}-form` : ''} appeared!`;
    const padding = Math.floor((80 - defaultText.length) / 2);

    return boxen(
      defaultText,
      {
        borderStyle: 'double',
        padding: {
          top: 1,
          right: padding,
          bottom: 1,
          left: padding + (80 - (padding * 2 + defaultText.length)),
        },
        ...options,
      },
    );
  },
};
