const requireAll = requireContext =>
  requireContext.keys().map(
    key => {
      const details = key.replace('./', '').replace('.png', '').split('-');
      const [number, pokemon, form] = details;
      return {
        number: parseInt(number, 10),
        pokemon,
        form,
        say: requireContext(key),
      };
    }
  );

module.exports = requireAll(require.context('../src/images/', true, /^\.\/.*\.png$/));
