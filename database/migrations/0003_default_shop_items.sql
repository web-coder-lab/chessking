-- Chess King — Migration 0003: default shop items + gift catalog
-- Doc 6 §2.3: every new user gets one free pre-equipped default item per
-- category (board/piece_set/avatar/banner). §3.1: gift items are shop_items
-- with category='gift', priced in coins, not equippable.
-- Additive only, per Doc 1's "never modify a table, always new migration."

INSERT INTO shop_items (id, category, name, description, image_url, price_coins, is_active, is_limited_edition) VALUES
    ('item_default_board',    'board',     'Classic Board',   'The standard Chess King board.',     '/assets/boards/classic.png',     0, 1, 0),
    ('item_default_pieces',   'piece_set', 'Classic Pieces',  'The standard staunton-style set.',   '/assets/pieces/classic.png',     0, 1, 0),
    ('item_default_avatar',   'avatar',    'Default Avatar',  'Starting profile avatar.',            '/assets/avatars/default.png',    0, 1, 0),
    ('item_default_banner',   'banner',    'Default Banner',  'Starting profile banner.',             '/assets/banners/default.png',    0, 1, 0);

-- §3.1 gift catalog - four tiers, Simple through VIP (5 to 2000 coins).
-- Every gift uses icon_emoji instead of image_url: this project has no
-- actual gift image assets, so the old rows referencing paths like
-- /assets/gifts/teddy.png would have rendered as broken images. Plain
-- Unicode glyphs render everywhere with zero asset files and zero
-- copyright concerns - nothing here is anyone else's artwork.
INSERT INTO shop_items (id, category, name, description, icon_emoji, price_coins, is_active, is_limited_edition) VALUES
    -- Simple
    ('gift_rose',        'gift', 'Rose',          'A small kind gesture.',            '🌹', 5,    1, 0),
    ('gift_clap',        'gift', 'Applause',       'Nicely played!',                   '👏', 8,    1, 0),
    ('gift_heart',       'gift', 'Heart',          'Show some love.',                  '❤️', 10,   1, 0),
    ('gift_coffee',      'gift', 'Coffee',         'Fuel for the next match.',         '☕', 15,   1, 0),
    ('gift_teddy_bear',  'gift', 'Teddy Bear',     'Send a friendly teddy bear.',      '🧸', 20,   1, 0),
    -- Nice
    ('gift_balloon',     'gift', 'Balloon',        'A little celebration.',            '🎈', 30,   1, 0),
    ('gift_star',        'gift', 'Star',           'You shone out there.',             '⭐', 35,   1, 0),
    ('gift_pawn',        'gift', 'Golden Pawn',    'Every king starts somewhere.',     '♟️', 45,   1, 0),
    ('gift_fireworks',   'gift', 'Fireworks',      'Big win energy.',                  '🎆', 60,   1, 0),
    ('gift_medal',       'gift', 'Medal',          'A well-earned medal.',             '🥇', 80,   1, 0),
    -- Premium
    ('gift_trophy',      'gift', 'Trophy',         'Celebrate a great game.',          '🏆', 100,  1, 0),
    ('gift_bouquet',     'gift', 'Bouquet',        'A grand gesture.',                 '💐', 150,  1, 0),
    ('gift_diamond',     'gift', 'Diamond',        'Pure brilliance.',                 '💎', 250,  1, 0),
    ('gift_ring',        'gift', 'Ring',           'Reserved for the boldest moves.',  '💍', 300,  1, 0),
    -- VIP
    ('gift_crown',       'gift', 'Crown',          'For the true king.',               '👑', 500,  1, 0),
    ('gift_castle',      'gift', 'Castle',         'Own the whole board.',             '🏰', 800,  1, 0),
    ('gift_dragon',      'gift', 'Dragon',         'A legendary send.',                '🐉', 1200, 1, 0),
    ('gift_rocket',      'gift', 'Rocket',         'The ultimate flex.',               '🚀', 2000, 1, 0);
