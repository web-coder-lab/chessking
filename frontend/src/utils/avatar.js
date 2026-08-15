/** Map equipped avatar_id / banner_id to display glyph or image. */

const AVATAR_EMOJI = {
  item_default_avatar: '👤',
  item_avatar_crown: '👑',
  item_avatar_knight: '♞',
  item_avatar_fire: '🔥',
};

const BANNER_EMOJI = {
  item_default_banner: '🖼️',
  item_banner_gold: '🥇',
  item_banner_night: '🌙',
};

export function avatarEmoji(avatarId) {
  if (!avatarId) return '👤';
  return AVATAR_EMOJI[avatarId] || '👤';
}

export function bannerEmoji(bannerId) {
  if (!bannerId) return '🖼️';
  return BANNER_EMOJI[bannerId] || '🖼️';
}

export function avatarImageUrl(user) {
  if (!user) return null;
  if (user.avatarUrl && (user.avatarUrl.startsWith('http') || user.avatarUrl.startsWith('data:'))) {
    return user.avatarUrl;
  }
  return null;
}
