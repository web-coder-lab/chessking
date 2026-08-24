import { useEffect, useRef, useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import TopBar from '../../components/navigation/TopBar';
import Button from '../../components/common/Button';
import Input from '../../components/common/Input';
import Toast from '../../components/common/Toast';
import { walletApi } from '../../services/api';
import './Checkout.css';

const GATEWAYS = [
  { id: 'jazzcash', label: 'JazzCash', icon: '📱' },
  { id: 'easypaisa', label: 'EasyPaisa', icon: '💳' },
  { id: 'googlepay', label: 'Google Pay', icon: '🅖' },
];

const POLL_INTERVAL_MS = 3000;

export default function Checkout({ user }) {
  const navigate = useNavigate();
  const location = useLocation();
  const preselected = location.state?.amountPkr;

  const [amountPkr, setAmountPkr] = useState(preselected ? String(preselected) : '');
  const [gateway, setGateway] = useState(null);
  const [amountError, setAmountError] = useState('');
  const [phone, setPhone] = useState('');
  const [phoneError, setPhoneError] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [toast, setToast] = useState(null);
  const [phase, setPhase] = useState('form');
  const [coinsCredited, setCoinsCredited] = useState(null);
  const pollTimer = useRef(null);

  useEffect(() => () => clearTimeout(pollTimer.current), []);

  function validateAmount(raw) {
    const n = Number(raw);
    if (!raw || !Number.isInteger(n) || n <= 0) return 'Enter a valid amount';
    if (n < 100) return 'Minimum deposit is Rs 100';
    return '';
  }

  function validatePhone(raw) {
    const digits = raw.replace(/\D/g, '');
    const looksValid =
      (digits.length === 11 && digits.startsWith('03')) ||
      (digits.length === 12 && digits.startsWith('923'));
    return looksValid ? '' : 'Enter a valid number, e.g. 03XXXXXXXXX';
  }

  const requiresPhone = gateway === 'jazzcash' || gateway === 'easypaisa';

  async function handlePay() {
    const err = validateAmount(amountPkr);
    setAmountError(err);
    if (err) return;
    if (!gateway) {
      setToast({ message: 'Choose a payment method' });
      return;
    }
    if (requiresPhone) {
      const pErr = validatePhone(phone);
      setPhoneError(pErr);
      if (pErr) return;
    }

    setSubmitting(true);
    try {
      const resp = await walletApi.initiateDeposit(
        Number(amountPkr),
        gateway,
        requiresPhone ? phone : undefined
      );
      setPhase('pending');
      if (resp.redirect_url) {
        window.open(resp.redirect_url, '_blank', 'noopener,noreferrer');
      }
      pollStatus(resp.payment_transaction_id);
    } catch (e) {
      if (e.code === 'duplicate_request') {
        setToast({ message: 'That deposit is already in progress' });
      } else {
        setToast({ message: e.message || 'Could not start payment' });
      }
    } finally {
      setSubmitting(false);
    }
  }

  function pollStatus(transactionId) {
    pollTimer.current = setTimeout(async () => {
      try {
        const s = await walletApi.getDepositStatus(transactionId);
        if (s.status === 'success') {
          setCoinsCredited(s.coins_credited);
          setPhase('success');
        } else if (s.status === 'failed') {
          setPhase('failed');
        } else {
          pollStatus(transactionId);
        }
      } catch {
        pollStatus(transactionId);
      }
    }, POLL_INTERVAL_MS);
  }

  return (
    <div className="ck-checkout">
      <TopBar
        avatarUser={user}
        coinBalance={user?.coin_balance ?? 0}
        onBellClick={() => navigate('/notifications')}
      />

      <main className="ck-checkout__body">
        <h1 className="page-title">Add Coins</h1>

        {phase === 'form' && (
          <>
            <div className="ck-checkout__amount-card">
              <span className="text-secondary">Amount (PKR)</span>
              <Input
                id="amount"
                type="number"
                inputMode="numeric"
                placeholder="e.g. 500"
                value={amountPkr}
                onChange={(e) => {
                  setAmountPkr(e.target.value);
                  setAmountError('');
                }}
                error={amountError}
                disabled={!!preselected}
              />
            </div>

            <section>
              <p className="ck-checkout__section-title">Payment method</p>
              <div className="ck-checkout__gateway-list">
                {GATEWAYS.map((g) => (
                  <button
                    type="button"
                    key={g.id}
                    className={`ck-checkout__gateway-row ${
                      gateway === g.id ? 'ck-checkout__gateway-row--selected' : ''
                    }`}
                    onClick={() => setGateway(g.id)}
                  >
                    <span aria-hidden="true">{g.icon}</span>
                    <span>{g.label}</span>
                    {gateway === g.id && (
                      <span className="ck-checkout__check" aria-hidden="true">
                        ✓
                      </span>
                    )}
                  </button>
                ))}
              </div>
            </section>

            {requiresPhone && (
              <div className="ck-checkout__amount-card">
                <span className="text-secondary">
                  {gateway === 'jazzcash' ? 'JazzCash' : 'EasyPaisa'} number
                </span>
                <Input
                  id="payer-phone"
                  type="tel"
                  inputMode="numeric"
                  placeholder="03XXXXXXXXX"
                  value={phone}
                  onChange={(e) => {
                    setPhone(e.target.value);
                    setPhoneError('');
                  }}
                  error={phoneError}
                />
              </div>
            )}

            <Button
              className="ck-checkout__pay"
              onClick={handlePay}
              loading={submitting}
              loadingLabel="Starting…"
            >
              Pay now
            </Button>
          </>
        )}

        {phase === 'pending' && (
          <div className="ck-checkout__status">
            <span className="ck-spinner ck-spinner--lg" aria-hidden="true" />
            <p>Waiting for payment confirmation…</p>
            <p className="ck-checkout__status-hint">
              Complete payment in the opened window. This updates automatically.
            </p>
          </div>
        )}

        {phase === 'success' && (
          <div className="ck-checkout__status">
            <span className="ck-checkout__status-icon" aria-hidden="true">
              🎉
            </span>
            <p>
              {coinsCredited != null
                ? `${coinsCredited.toLocaleString()} coins added!`
                : 'Payment confirmed!'}
            </p>
            <Button onClick={() => navigate('/wallet')}>Back to wallet</Button>
          </div>
        )}

        {phase === 'failed' && (
          <div className="ck-checkout__status">
            <span className="ck-checkout__status-icon" aria-hidden="true">
              ⚠️
            </span>
            <p>Payment didn&apos;t go through.</p>
            <Button onClick={() => setPhase('form')}>Try again</Button>
          </div>
        )}
      </main>

      <Toast visible={!!toast} message={toast?.message} onDismiss={() => setToast(null)} />
    </div>
  );
}
