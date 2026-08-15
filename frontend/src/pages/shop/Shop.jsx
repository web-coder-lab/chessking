import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import TopBar from '../../components/navigation/TopBar';
import BottomNav from '../../components/navigation/BottomNav';
import Card from '../../components/common/Card';
import Button from '../../components/common/Button';
import Toast from '../../components/common/Toast';
import { shopApi } from '../../services/api';
import ItemVisual from '../../components/shop/ItemVisual';
import './Shop.css';

const CATEGORIES = [
  { key: 'board', label: 'Boards' },
  { key: 'piece_set', label: 'Pieces' },
  { key: 'avatar', label: 'Avatars' },
  { key: 'banner', label: 'Banners' },
];

export default function Shop({ user }) {
  const navigate = useNavigate();
  const [category, setCategory] = useState('board');
  const [items, setItems] = useState([]);
  const [previewItem, setPreviewItem] = useState(null);
  const [toast, setToast] = useState(null);
  const [balance, setBalance] = useState(user?.coin_balance ?? 0);

  useEffect(() => {
    shopApi.listItems(category).then((res) => setItems(res.items));
  }, [category]);

  async function handleBuy(item) {
    try {
      const resp = await shopApi.purchase(item.id);
      setBalance(resp.new_balance);
      setItems((prev) => prev.map((i) => (i.id === item.id ? { ...i, owned: true } : i)));
      setPreviewItem(null);
    } catch (err) {
      if (err.code === 'insufficient_coins') {
        // §2.5: "small inline toast 'Not enough coins' + shortcut button to Wallet"
        setToast({ message: 'Not enough coins', actionLabel: 'Add Coins', action: () => navigate('/wallet') });
      } else {
        setToast({ message: err.message });
      }
    }
  }

  return (
    <div className="ck-shop">
      <TopBar avatarUser={user} coinBalance={balance} onBellClick={() => navigate('/notifications')} />

      <main className="ck-shop__body">
        {/* §2.5: horizontal tab/segment switcher */}
        <div className="ck-shop__segments" role="tablist">
          {CATEGORIES.map((c) => (
            <button
              key={c.key}
              role="tab"
              aria-selected={category === c.key}
              className={`ck-shop__segment ${category === c.key ? 'ck-shop__segment--active' : ''}`}
              onClick={() => setCategory(c.key)}
            >
              {c.label}
            </button>
          ))}
        </div>

        {/* §2.5: 2-column grid */}
        <div className="ck-shop__grid">
          {items.map((item) => (
            <Card key={item.id} className="ck-shop__item-card" onClick={() => setPreviewItem(item)}>
              {item.owned && <span className="ck-shop__owned-ribbon">Owned</span>}
              <ItemVisual item={item} className="ck-shop__item-image" />
              <span className="ck-shop__item-name">{item.name}</span>
              <span className="ck-shop__item-price tabular-nums">🪙 {item.price_coins}</span>
              {!item.owned && (
                <Button
                  fullWidth
                  onClick={(e) => { e.stopPropagation(); handleBuy(item); }}
                >
                  Buy
                </Button>
              )}
            </Card>
          ))}
        </div>
      </main>

      {/* §2.5: tap card (not buy button) -> preview modal */}
      {previewItem && (
        <div className="ck-shop__modal-overlay" onClick={() => setPreviewItem(null)}>
          <div className="ck-shop__modal" onClick={(e) => e.stopPropagation()}>
            <ItemVisual item={previewItem} className="ck-shop__modal-image" />
            <h2 className="section-heading">{previewItem.name}</h2>
            <p className="text-secondary">{previewItem.description}</p>
            <p className="ck-shop__modal-price tabular-nums">🪙 {previewItem.price_coins}</p>
            {previewItem.owned ? (
              <span className="ck-shop__owned-ribbon" style={{ position: 'static' }}>Owned</span>
            ) : (
              <Button onClick={() => handleBuy(previewItem)}>Buy</Button>
            )}
          </div>
        </div>
      )}

      <Toast
        visible={!!toast}
        message={toast?.message}
        actionLabel={toast?.actionLabel}
        onAction={toast?.action}
        onDismiss={() => setToast(null)}
      />

      <BottomNav />
    </div>
  );
}
