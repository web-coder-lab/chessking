import { Component } from 'react';

export default class ErrorBoundary extends Component {
  constructor(props) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error) {
    return { error };
  }

  componentDidCatch(error, info) {
    console.error('GeniusClan UI crash', error, info);
  }

  render() {
    if (this.state.error) {
      return (
        <div style={{ minHeight: '100dvh', background: '#0F1115', color: '#F5F5F5', padding: 24, fontFamily: 'system-ui, sans-serif', textAlign: 'center' }}>
          <div style={{ fontSize: 40, marginBottom: 8 }}>♚</div>
          <p style={{ color: '#D4AF37', letterSpacing: 3, textTransform: 'uppercase', fontSize: 12 }}>Genius Clan</p>
          <h1 style={{ fontSize: 20 }}>Something went wrong</h1>
          <p style={{ color: '#9CA3AF', fontSize: 14, marginBottom: 24 }}>{String(this.state.error?.message || this.state.error)}</p>
          <button
            type="button"
            onClick={() => {
              try { localStorage.removeItem('ck_refresh_token'); } catch (_) {}
              window.location.href = '/auth';
            }}
            style={{ background: '#D4AF37', color: '#0F1115', border: 'none', padding: '12px 20px', borderRadius: 8, fontWeight: 700, cursor: 'pointer' }}
          >
            Clear session & sign in
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
