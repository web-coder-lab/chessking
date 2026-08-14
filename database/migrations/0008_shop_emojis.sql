
UPDATE shop_items SET icon_emoji = '♟️', description = 'The standard Genius Clan board.' WHERE id = 'item_default_board';
UPDATE shop_items SET icon_emoji = '♜' WHERE id = 'item_default_pieces';
UPDATE shop_items SET icon_emoji = '👤' WHERE id = 'item_default_avatar';
UPDATE shop_items SET icon_emoji = '🖼️' WHERE id = 'item_default_banner';

INSERT OR IGNORE INTO shop_items (id, category, name, description, image_url, icon_emoji, price_coins, is_active, is_limited_edition) VALUES
 ('item_avatar_crown', 'avatar', 'Crown Avatar', 'Royal gold crown style.', NULL, '👑', 50, 1, 0),
 ('item_avatar_knight', 'avatar', 'Knight Avatar', 'Knight-themed profile.', NULL, '♞', 40, 1, 0),
 ('item_avatar_fire', 'avatar', 'Fire Avatar', 'Blazing competitor look.', NULL, '🔥', 60, 1, 0),
 ('item_banner_gold', 'banner', 'Gold Banner', 'Gold gradient profile banner.', NULL, '🥇', 80, 1, 0),
 ('item_banner_night', 'banner', 'Night Banner', 'Dark night sky banner.', NULL, '🌙', 70, 1, 0),
 ('item_board_marble', 'board', 'Marble Board', 'Elegant marble squares.', NULL, '⬜', 100, 1, 0),
 ('item_pieces_gold', 'piece_set', 'Gold Pieces', 'Shiny gold piece set.', NULL, '♚', 120, 1, 0);
