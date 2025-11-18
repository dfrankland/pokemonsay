SELECT 'ALTER TABLE pokemon_v2_pokemonspecies DROP COLUMN "' || name || '";'
FROM pragma_table_info('pokemon_v2_pokemonspecies')
WHERE name != 'id';
