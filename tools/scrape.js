import fetch from 'node-fetch';
import cheerio from 'cheerio';
import { parse as parseUrl } from 'url';
import { createWriteStream } from 'fs';
import { resolve as resolvePath } from 'path';
import ProgressBar from 'progress';

const LIST_OF_POKEMON =
  'http://bulbapedia.bulbagarden.net/wiki/List_of_Pok%C3%A9mon_by_National_Pok%C3%A9dex_number';
const IMAGE_SELECTOR = '#mw-content-text table tr td a img';

(async () => {
  const response = await fetch(LIST_OF_POKEMON);
  const html = await response.text();
  const $ = cheerio.load(html);
  const images = $(IMAGE_SELECTOR)
    .map(
      (index, element) => {
        const src = $(element).attr('src');
        const file = parseUrl(src).pathname.split('/').pop();
        const number = file.match(/^[0-9]*/)[0];
        const form = file.replace(number, '').replace('MS.png', '');
        const pokemon = $(element).attr('alt');
        const filename = `${number}-${pokemon}${form ? `-${form}` : ''}.png`;
        return {
          src,
          filename,
          pokemon,
        };
      }
    ).get();
  const bar = new ProgressBar('Image files downloaded [:bar] (:current/:total) :percent :etas', {
    complete: '=',
    incomplete: ' ',
    width: 80,
    total: images.length,
  });
  images.forEach(
    async ({ src, pokemon, filename }) => {
      const { body } = await fetch(src);
      body.on('end', () => {
        bar.tick();
      });
      body.on('error', err => {
        console.error(`Couldn't download "${pokemon}" from "${src}"`);
        console.error(err);
      });
      body.pipe(
        createWriteStream(
          resolvePath(__dirname, `../src/images/${filename}`)
        )
      );
    }
  );
})().catch(err => {
  console.error(err);
  process.exit(1);
});
