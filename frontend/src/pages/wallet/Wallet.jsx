import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import TopBar from '../../components/navigation/TopBar';
import BottomNav from '../../components/navigation/BottomNav';
import Card from '../../components/common/Card';
import Button from '../../components/common/Button';
import EmptyState from '../../components/common/EmptyState';
import { walletApi } from '../../services/api';
import './Wallet.css';

const TYPE_ICON = {
  wallet_plus: '💰', bag: '🛍️', gift: '🎁', calendar: '📅',
  play_video: '▶️', people: '👥', shield: '🛡️', undo: '↩️',
};

export default function WalletScreen({ user }) {
  const navigate = useNavigate();
  const [balance, setBalance] = useState(user?.coin_balance ?? 0);
  const [packages, setPackages] = useState([]);
  const [transactions, setTransactions] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      walletApi.getBalance(),
      walletApi.getPackages(),   // §1: packages come from backend, never hardcoded
      walletApi.getTransactions(),
    ]).then(([b, p, t]) => {
      setBalance(b.coin_balance);
      setPackages(p.packages);
      setTransactions(t.transactions);
    }).finally(() => setLoading(false));
  }, []);

  async function handleBuyPackage(pkg) {
    // §2 step 1-3: select package -> select gateway -> POST initiate.
    // Gateway selection sheet omitted here for brevity; wired the same
    // way once the Custom Amount + gateway-picker sheet is built.
    navigate('/wallet/checkout', { state: { amountPkr: pkg.amount_pkr } });
  }

  return (
    <div className="ck-wallet">
      <TopBar avatarUrl={user?.avatarUrl} coinBalance={balance} onBellClick={() => navigate('/notifications')} />

      <main className="ck-wallet__body">
        {/* §2.4: large balance card, gold gradient, full-width */}
        <div className="ck-wallet__balance-card">
          <span className="ck-wallet__balance-subtitle">Available Balance</span>
          <div className="ck-wallet__balance-amount">
            <span aria-hidden="true">🪙</span>
            <span className="tabular-nums">{balance.toLocaleString()}</span>
          </div>
        </div>

        <Button variant="outline" onClick={() => navigate('/wallet/checkout')}>
          Add Coins
        </Button>

        {/* §2.4: coin package grid, 2 columns, bonus ribbon */}
        <div className="ck-wallet__package-grid">
          {packages.map((pkg) => (
            <Card key={pkg.id} className="ck-wallet__package-card" onClick={() => handleBuyPackage(pkg)}>
              {pkg.bonus_label && <span className="ck-wallet__bonus-ribbon">{pkg.bonus_label}</span>}
              <span className="ck-wallet__package-coins tabular-nums">🪙 {pkg.coins.toLocaleString()}</span>
              <span className="ck-wallet__package-price text-secondary">Rs {pkg.amount_pkr.toLocaleString()}</span>
            </Card>
          ))}
        </div>

        <section>
          <h2 className="section-heading" style={{ marginBottom: 'var(--space-3)' }}>
            Transaction History
          </h2>

          {!loading && transactions.length === 0 && (
            <EmptyState icon="💳" text="No transactions yet" />
          )}

          <div className="ck-wallet__tx-list">
            {transactions.map((tx) => (
              <Card key={tx.id} className="ck-wallet__tx-row">
                <span className="ck-wallet__tx-icon" aria-hidden="true">{TYPE_ICON[tx.icon] ?? '💰'}</span>
                <div className="ck-wallet__tx-info">
                  <span className="ck-wallet__tx-label">{tx.label}</span>
                  <span className="ck-wallet__tx-time text-secondary">{formatRelativeTime(tx.created_at)}</span>
                </div>
                <span className={`ck-wallet__tx-amount tabular-nums ${tx.amount >= 0 ? 'ck-wallet__tx-amount--positive' : 'ck-wallet__tx-amount--negative'}`}>
                  {tx.amount >= 0 ? '+' : ''}{tx.amount}
                </span>
              </Card>
            ))}
          </div>
        </section>
      </main>

      <BottomNav />
    </div>
  );
}

function formatRelativeTime(iso) {
  const diffMs = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diffMs / 60000);
  if (mins < 1) return 'Just now';
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  return `${Math.floor(hrs / 24)}d ago`;
}
