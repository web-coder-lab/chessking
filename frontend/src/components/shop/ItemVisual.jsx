/** Shows emoji or image for shop/inventory items — never a broken image box. */
const FALLBACK = {
  board: '♟️',
  piece_set: '♜',
  avatar: '👤',
  banner: '🖼️',
  gift: '🎁',
};

export default function ItemVisual({ item, className = '' }) {
  const emoji = item?.icon_emoji || FALLBACK[item?.category] || '✨';
  const src = item?.image_url;

  if (src && (src.startsWith('http://') || src.startsWith('https://') || src.startsWith('data:'))) {
    return <img src={src} alt="" className={className} />;
  }

  return (
    <div
      className={className}
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: '2.5rem',
        width: '100%',
        minHeight: 72,
        background: 'linear-gradient(145deg, #1A1D23, #252830)',
        borderRadius: 12,
      }}
      aria-hidden
    >
      {emoji}
    </div>
  );
}
