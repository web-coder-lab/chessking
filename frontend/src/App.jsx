import { BrowserRouter, Routes, Route, Navigate, useNavigate } from 'react-router-dom';
import { AuthProvider, useAuth } from './context/AuthContext';
import Splash from './pages/splash/Splash';
import AuthScreen from './pages/auth/AuthScreen';
import ResetPasswordScreen from './pages/auth/ResetPasswordScreen';
import CompleteSignup from './pages/auth/CompleteSignup';
import VerifyEmail from './pages/auth/VerifyEmail';
import Dashboard from './pages/dashboard/Dashboard';
import WalletScreen from './pages/wallet/Wallet';
import Checkout from './pages/wallet/Checkout';
import Shop from './pages/shop/Shop';
import Inventory from './pages/inventory/Inventory';
import Play from './pages/play/Play';
import ChessBoard from './pages/board/ChessBoard';
import Leaderboard from './pages/leaderboard/Leaderboard';
import Profile from './pages/profile/Profile';
import ProfileSettings from './pages/profile/ProfileSettings';
import Settings from './pages/settings/Settings';
import TwoFactorSettings from './pages/settings/TwoFactorSettings';
import SessionsSettings from './pages/settings/SessionsSettings';
import StaticContent from './pages/settings/StaticContent';
import SupportPage from './pages/settings/SupportPage';
import BugReport from './pages/settings/BugReport';
import { supportApi } from './services/api';
import InviteFriend from './pages/invite/InviteFriend';
import CustomMatch from './pages/custom-match/CustomMatch';
import NotificationsDrawer from './components/notifications/NotificationsDrawer';
import NotFound from './pages/not-found/NotFound';
import './styles/tokens.css';

/**
 * Doc 3 §8: frontend route guards are UX-only convenience — never the
 * real security boundary, since every backend endpoint independently
 * re-validates the JWT on every request regardless of what this shows.
 */
function ProtectedRoute({ children }) {
  const { isAuthenticated, bootstrapping } = useAuth();
  if (bootstrapping) {
    return (
      <div style={{ minHeight: '100dvh', background: '#0F1115', color: '#D4AF37', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        ♚ Loading…
      </div>
    );
  }
  if (!isAuthenticated) return <Navigate to="/auth" replace />;
  return children;
}

function NotificationsPage() {
  const navigate = useNavigate();
  return <NotificationsDrawer open onClose={() => navigate(-1)} />;
}

function AppRoutes() {
  const { user, refreshUser } = useAuth();
  return (
    <Routes>
      <Route path="/" element={<Splash />} />
      <Route path="/auth" element={<AuthScreen />} />
      <Route path="/reset-password" element={<ResetPasswordScreen />} />
      <Route path="/complete-signup" element={<CompleteSignup />} />
      <Route path="/verify-email" element={<VerifyEmail />} />

      <Route path="/dashboard" element={<ProtectedRoute><Dashboard user={user} refreshUser={refreshUser} /></ProtectedRoute>} />
      <Route path="/wallet" element={<ProtectedRoute><WalletScreen user={user} /></ProtectedRoute>} />
      <Route path="/wallet/checkout" element={<ProtectedRoute><Checkout user={user} /></ProtectedRoute>} />
      <Route path="/shop" element={<ProtectedRoute><Shop user={user} /></ProtectedRoute>} />
      <Route path="/inventory" element={<ProtectedRoute><Inventory user={user} refreshUser={refreshUser} /></ProtectedRoute>} />
      <Route path="/play" element={<ProtectedRoute><Play user={user} /></ProtectedRoute>} />
      <Route path="/board/:matchId" element={<ProtectedRoute><ChessBoard user={user} /></ProtectedRoute>} />
      <Route path="/leaderboard" element={<ProtectedRoute><Leaderboard user={user} /></ProtectedRoute>} />
      <Route path="/profile" element={<ProtectedRoute><Profile user={user} /></ProtectedRoute>} />
      <Route path="/profile/settings" element={<ProtectedRoute><ProfileSettings user={user} /></ProtectedRoute>} />
      <Route path="/profile/:username" element={<ProtectedRoute><Profile user={user} /></ProtectedRoute>} />
      <Route path="/settings" element={<ProtectedRoute><Settings user={user} /></ProtectedRoute>} />
      <Route path="/settings/2fa" element={<ProtectedRoute><TwoFactorSettings user={user} refreshUser={refreshUser} /></ProtectedRoute>} />
      <Route path="/settings/sessions" element={<ProtectedRoute><SessionsSettings /></ProtectedRoute>} />
      <Route path="/settings/bug-report" element={<ProtectedRoute><BugReport /></ProtectedRoute>} />
      <Route path="/settings/support" element={<ProtectedRoute><SupportPage /></ProtectedRoute>} />
      <Route path="/settings/privacy-policy" element={<StaticContent title="Privacy Policy" fetchFn={supportApi.getPrivacyPolicy} />} />
      <Route path="/settings/terms-of-service" element={<StaticContent title="Terms of Service" fetchFn={supportApi.getTermsOfService} />} />
      <Route path="/settings/about" element={<StaticContent title="About" fetchFn={supportApi.getAbout} />} />
      <Route path="/invite" element={<ProtectedRoute><InviteFriend /></ProtectedRoute>} />
      <Route path="/custom-match" element={<ProtectedRoute><CustomMatch user={user} /></ProtectedRoute>} />
      <Route path="/notifications" element={<ProtectedRoute><NotificationsPage /></ProtectedRoute>} />

      <Route path="*" element={<NotFound />} />
    </Routes>
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <AuthProvider>
        <AppRoutes />
      </AuthProvider>
    </BrowserRouter>
  );
}
