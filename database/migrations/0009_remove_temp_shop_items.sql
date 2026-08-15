-- Part 14: remove temporary avatar/banner/board SKUs added in 0008
DELETE FROM shop_items WHERE id IN (
  'item_avatar_crown',
  'item_avatar_knight',
  'item_avatar_fire',
  'item_banner_gold',
  'item_banner_night',
  'item_board_marble',
  'item_pieces_gold'
);

-- Clean default copy + emoji (Genius Clan)
UPDATE shop_items SET
  name = 'Classic Board',
  description = 'The standard Genius Clan board.',
  icon_emoji = '♟️'
WHERE id = 'item_default_board';

UPDATE shop_items SET
  name = 'Classic Pieces',
  description = 'Standard staunton-style pieces.',
  icon_emoji = '♜'
WHERE id = 'item_default_pieces';

UPDATE shop_items SET
  name = 'Default Avatar',
  description = 'Starting profile avatar.',
  icon_emoji = '👤'
WHERE id = 'item_default_avatar';

UPDATE shop_items SET
  name = 'Default Banner',
  description = 'Starting profile banner.',
  icon_emoji = '🖼️'
WHERE id = 'item_default_banner';
