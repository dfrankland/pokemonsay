import webpack from 'webpack';
import fs from 'fs';
import MemoryFileSystem from 'memory-fs';
import requireFromString from 'require-from-string';
import { resolve as resolvePath } from 'path';
import webpackConfig from './webpack.config';

const fsMemory = new MemoryFileSystem();

const compiler = webpack(webpackConfig);
compiler.outputFileSystem = fsMemory;
compiler.plugin(
  'after-emit',
  (compilation, callback) => {
    Object.keys(compilation.assets).forEach(
      outname => {
        if (compilation.assets[outname].emitted) {
          const path = fsMemory.join(compiler.outputPath, outname);
          const string = fsMemory.readFileSync(path, 'utf8');
          const pokemonJson = requireFromString(string);
          fs.writeFile(
            resolvePath(__dirname, '../dist/pokemon.json'),
            JSON.stringify(pokemonJson),
            (err) => {
              if (err) console.error(err);
            }
          );
        }
      }
    );
    callback();
  },
);

compiler.run(
  (err, stats) => {
    if (err) {
      console.error(err);
    }

    stats.toString(webpackConfig.stats);
  }
);
