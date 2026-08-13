import { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import TopBar from '../../components/navigation/TopBar';
import BottomNav from '../../components/navigation/BottomNav';
import Card from '../../components/common/Card';
import Button from '../../components/common/Button';
import EmptyState from '../../components/common/EmptyState';
import { inventoryApi } from '../../services/api';
import './Inventory.css';

const CATEGORIES = [
  { key: 'board', label: 'Boards' },
  { key: 'piece_set', label: 'Pieces' },
  { key: 'avatar', label: 'Avatars' },
  { key: 'banner', label: 'Banners' },
];

export default function Inventory({ user }) {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const requestedCategory = searchParams.get('category');
  const initialCategory = CATEGORIES.some((c) => c.key === requestedCategory) ? requestedCategory : 'board';
  const [category, setCategory] = useState(initialCategory);
  const [items, setItems] = useState([]);
  const [sheetItem, setSheetItem] = useState(null);

  useEffect(() => {
    inventoryApi.list().then((res) => setItems(res.items));
  }, []);

  const visible = items.filter((i) => i.category === category);

  async function handleToggleEquip(item) {
    // §2.2: equipping auto-unequips the previous item in the same
    // category — UI reflects this instantly.
    await inventoryApi.equip(item.inventory_id);
    setItems((prev) =>
      prev.map((i) => {
        if (i.category !== item.category) return i;
        return { ...i, is_equipped: i.inventory_id === item.inventory_id ? 1 : 0 };
      })
    );
    setSheetItem(null);
  }

  return (
    <div className="ck-inventory">
      <TopBar avatarUrl={user?.avatarUrl} coinBalance={user?.coin_balance ?? 0} onBellClick={() => navigate('/notifications')} />

      <main className="ck-inventory__body">
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

        {visible.length === 0 ? (
          <EmptyState icon="🎒" text="No items in this category yet" />
        ) : (
          <div className="ck-shop__grid">
            {visible.map((item) => (
              <Card
                key={item.inventory_id}
                className={`ck-inventory__item-card ${item.is_equipped ? 'ck-inventory__item-card--equipped' : ''}`}
                onClick={() => setSheetItem(item)}
              >
                {item.is_equipped === 1 && <span className="ck-inventory__equipped-label">Equipped</span>}
                <img src={item.image_url} alt="" className="ck-shop__item-image" />
                <span className="ck-shop__item-name">{item.name}</span>
              </Card>
            ))}
          </div>
        )}
      </main>

      {/* §2.6: tap card -> bottom sheet. Note: §2.2 explicitly forbids a
          zero-equipped state once an item has been equipped, so there is
          no standalone "unequip to nothing" backend action — equipping a
          DIFFERENT item in the category is the only way to change what's
          equipped. The sheet reflects that: already-equipped items show
          a disabled confirmation, not an Unequip button that would leave
          the category empty. */}
      {sheetItem && (
        <div className="ck-inventory__sheet-overlay" onClick={() => setSheetItem(null)}>
          <div className="ck-inventory__sheet" onClick={(e) => e.stopPropagation()}>
            <img src={sheetItem.image_url} alt="" className="ck-inventory__sheet-image" />
            <h2 className="section-heading">{sheetItem.name}</h2>
            {sheetItem.is_equipped ? (
              <Button disabled pill>Currently Equipped</Button>
            ) : (
              <Button onClick={() => handleToggleEquip(sheetItem)} pill>Equip</Button>
            )}
          </div>
        </div>
      )}

      <BottomNav />
    </div>
  );
}
