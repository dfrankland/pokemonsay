import pokemonsay from './index';

const isBeingPipedInto = !process.stdin.isTTY;

const { pokemon, form, say } = pokemonsay.random();

const callback = text => {
  const box = pokemonsay.say({ pokemon, form, text });
  console.log(`${say}\n${box}`);
};

if (isBeingPipedInto) {
  let data = '';
  process.stdin.resume();
  process.stdin.setEncoding('utf8');
  process.stdin.on('data', chunk => data += chunk);
  process.stdin.on('end', () => {
    callback(data);
  });
} else {
  callback();
};
