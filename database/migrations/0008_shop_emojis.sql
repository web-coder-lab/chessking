-- Defaults emoji only (extra SKUs removed in 0009 / never re-added)
UPDATE shop_items SET icon_emoji = '♟️', description = 'The standard Genius Clan board.' WHERE id = 'item_default_board';
UPDATE shop_items SET icon_emoji = '♜' WHERE id = 'item_default_pieces';
UPDATE shop_items SET icon_emoji = '👤' WHERE id = 'item_default_avatar';
UPDATE shop_items SET icon_emoji = '🖼️' WHERE id = 'item_default_banner';
