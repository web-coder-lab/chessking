import { API_BASE } from '../config/endpoints.js';

// Simple in-memory holder so AuthContext can push the current access
// token here without every caller having to thread it through props.
let currentAccessToken = null;
export function setCurrentAccessToken(token) {
  currentAccessToken = token;
}


/** User-facing copy for known API / network codes */
export function friendlyError(err) {
  const code = err?.code || '';
  const status = err?.status;
  const map = {
    network_error: 'Server unreachable. Wait a few seconds (free server may be waking up) and try again.',
    email_send_failed: 'Email could not be sent. Check your inbox later or try Resend.',
    invalid_credentials: 'Wrong username/email or password.',
    username_taken: 'This username is already taken.',
    email_taken: 'This email is already registered.',
    username_invalid: 'Username must be 3–20 letters, numbers, or underscore.',
    password_weak: 'Password is too weak. Use a stronger one.',
    rate_limited: 'Too many tries. Wait a minute and try again.',
    unauthorized: 'Session expired. Please sign in again.',
    already_claimed: 'Already claimed today.',
    insufficient_coins: 'Not enough coins.',
  };
  if (map[code]) return map[code];
  if (status === 429) return map.rate_limited;
  if (status === 502 || status === 503) return 'Server is busy or waking up. Try again in a moment.';
  return err?.message || 'Something went wrong. Please try again.';
}

function authHeaders() {
  return currentAccessToken ? { Authorization: `Bearer ${currentAccessToken}` } : {};
}

async function request(path, options = {}) {
  const { skipAuth, headers: extraHeaders, ...rest } = options;
  const headers = {
    'Content-Type': 'application/json',
    ...(skipAuth ? {} : authHeaders()),
    ...extraHeaders,
  };

  let res;
  try {
    res = await fetch(`${API_BASE}${path}`, { ...rest, headers });
  } catch (e) {
    // Browser "Failed to fetch" — CORS, offline, wrong host, or sleeping server
    const err = new Error(
      'Cannot reach server. Wait a few seconds (free server may be waking up) and try again.'
    );
    err.code = 'network_error';
    throw err;
  }

  const data = await res.json().catch(() => ({}));

  if (!res.ok) {
    const code = (data.error && data.error.code) || data.code || 'error';
    const raw =
      (data.error && typeof data.error === 'object' && data.error.message) ||
      (typeof data.error === 'string' ? data.error : null) ||
      data.message ||
      '';
    const err = new Error(raw || 'Something went wrong.');
    err.code = code;
    err.status = res.status;
    err.message = friendlyError(err);
    throw err;
  }
  return data;
}

function deviceContext() {
  // Minimal device fingerprint / context sent with every auth call,
  // per Doc 3 §9 "Device fingerprinting recorded on every session".
  return {
    device_fingerprint: localStorage.getItem('ck_device_id') || cryptoRandomId(),
    browser: navigator.userAgent,
    os: navigator.platform,
  };
}

function cryptoRandomId() {
  const id = crypto.randomUUID();
  localStorage.setItem('ck_device_id', id);
  return id;
}

export const captchaApi = {
  generate: () => request('/captcha/generate', { method: 'POST', body: '{}' }),
  verify: (challenge_id, answer) =>
    request('/captcha/verify', { method: 'POST', body: JSON.stringify({ challenge_id, answer }) }),
};

export const authApi = {
  register: (username, email, password) =>
    request('/auth/register', { method: 'POST', body: JSON.stringify({ username, email, password, ...deviceContext() }) }),

  /** Part 10: email-only → complete-signup link */
  registerIntent: (email) =>
    request('/auth/register-intent', { method: 'POST', body: JSON.stringify({ email }), skipAuth: true }),

  completeSignup: (token, username, password) =>
    request('/auth/complete-signup', {
      method: 'POST',
      skipAuth: true,
      body: JSON.stringify({ token, username, password, ...deviceContext() }),
    }),

  verifyEmail: (token) =>
    request('/auth/verify-email', { method: 'POST', body: JSON.stringify({ token, ...deviceContext() }) }),

  resendVerification: (email) =>
    request('/auth/resend-verification', { method: 'POST', body: JSON.stringify({ email }) }),

  login: (identifier, password, captcha) =>
    request('/auth/login', { method: 'POST', body: JSON.stringify({
      identifier, password, ...deviceContext(),
      ...(captcha ? { captcha_challenge_id: captcha.challenge_id, captcha_answer: captcha.answer } : {}),
    }) }),

  // §5 Case C: only the already-logged-in OLD device calls this, so it's
  // a protected route (needs the caller's own access token) - it decides
  // approve/deny for a login attempt happening on a *different* device.
  respondDeviceApproval: (pendingId, decision) =>
    request('/auth/login/device-approval-response', {
      method: 'POST',
      headers: authHeaders(),
      body: JSON.stringify({ pending_id: pendingId, decision }), // decision: "approve" | "deny"
    }),

  submit2faCode: (pendingId, code) =>
    request('/auth/login/2fa', { method: 'POST', body: JSON.stringify({ pending_id: pendingId, code, ...deviceContext() }) }),

  checkDeviceApprovalStatus: (pendingId) =>
    request(`/auth/login/device-approval-status/${pendingId}`),

  refresh: (refreshToken) =>
    request('/auth/refresh', { method: 'POST', skipAuth: true, body: JSON.stringify({ refresh_token: refreshToken }) }),

  logout: (accessToken) =>
    request('/auth/logout', { method: 'POST', headers: { Authorization: `Bearer ${accessToken}` } }),

  forgotPassword: (email) =>
    request('/auth/forgot-password', { method: 'POST', body: JSON.stringify({ email }) }),

  resetPassword: (token, newPassword) =>
    request('/auth/reset-password', { method: 'POST', body: JSON.stringify({ token, new_password: newPassword }) }),

  enable2FA: (currentPassword, code, confirmCode) =>
    request('/auth/2fa/enable', { method: 'POST', headers: authHeaders(), body: JSON.stringify({ current_password: currentPassword, new_code: code, confirm_code: confirmCode }) }),

  disable2FA: (currentPassword, code) =>
    request('/auth/2fa/disable', { method: 'POST', headers: authHeaders(), body: JSON.stringify({ current_password: currentPassword, current_code: code }) }),

  getSessions: () => request('/auth/sessions', { headers: authHeaders() }),

  revokeSession: (sessionId) =>
    request(`/auth/sessions/${sessionId}`, { method: 'DELETE', headers: authHeaders() }),
};

// §8: every protected call carries the current access token — the
// backend independently re-validates it on every request regardless.


export const walletApi = {
  getBalance: () => request('/wallet/balance', { headers: authHeaders() }),

  // §1: packages come from the backend, never hardcoded in frontend code
  getPackages: () => request('/wallet/packages', { headers: authHeaders() }),

  getTransactions: () => request('/wallet/history', { headers: authHeaders() }),

  // §2 step 3: POST /wallet/deposit/initiate { amount_pkr, gateway, payer_phone? }
  initiateDeposit: (amountPkr, gateway, payerPhone) =>
    request('/wallet/deposit/initiate', {
      method: 'POST',
      headers: authHeaders(),
      body: JSON.stringify({
        amount_pkr: amountPkr,
        gateway,
        payer_phone: payerPhone || undefined,
        idempotency_key: crypto.randomUUID(), // §7: prevents double-tap duplicate orders
      }),
    }),

  // §2 step 9: frontend polls transaction status until success/failed
  getDepositStatus: (transactionId) =>
    request(`/wallet/deposit/${transactionId}/status`, { headers: authHeaders() }),
};

export const shopApi = {
  // §1.2/§1.4: items come from backend, availability-window-filtered server-side
  listItems: (category) =>
    request(`/shop/items${category ? `?category=${category}` : ''}`, { headers: authHeaders() }),

  purchase: (shopItemId) =>
    request('/shop/purchase', {
      method: 'POST',
      headers: authHeaders(),
      body: JSON.stringify({
        shop_item_id: shopItemId,
        idempotency_key: crypto.randomUUID(), // required - prevents double-tap double-charging
      }),
    }),
};

export const inventoryApi = {
  list: () => request('/inventory', { headers: authHeaders() }),

  equip: (inventoryId) =>
    request(`/inventory/${inventoryId}/equip`, { method: 'POST', headers: authHeaders() }),

  unequip: (inventoryId) =>
    request(`/inventory/${inventoryId}/unequip`, { method: 'POST', headers: authHeaders() }),
};

export const giftsApi = {
  getCatalog: () => request('/gifts/catalog', { headers: authHeaders() }),

  send: (receiverUsername, shopItemId, context, matchId) =>
    request('/gifts/send', {
      method: 'POST',
      headers: authHeaders(),
      body: JSON.stringify({ receiver_username: receiverUsername, shop_item_id: shopItemId, context, match_id: matchId }),
    }),

  receivedTally: (username) =>
    request(`/profile/${username}/gifts-received`, { headers: authHeaders() }),
};

export const socialApi = {
  getMyProfile: () => request('/profile/me', { headers: authHeaders() }),
  getPublicProfile: (username) => request(`/profile/${username}`, { headers: authHeaders() }),
  updateProfile: (body) => request('/profile/me', { method: 'PATCH', headers: authHeaders(), body: JSON.stringify(body) }),
  changePassword: (currentPassword, newPassword) =>
    request('/profile/me/password', { method: 'POST', headers: authHeaders(), body: JSON.stringify({ current_password: currentPassword, new_password: newPassword }) }),
  changeEmail: (currentPassword, newEmail) =>
    request('/profile/me/email', { method: 'POST', headers: authHeaders(), body: JSON.stringify({ current_password: currentPassword, new_email: newEmail }) }),
  getMatchHistory: (username, limit) =>
    request(`/profile/${username}/match-history${limit ? `?limit=${limit}` : ''}`, { headers: authHeaders() }),

  getReferralLink: () => request('/referral/link', { headers: authHeaders() }),
  getReferralProgress: () => request('/referral/progress', { headers: authHeaders() }),
  claimReferral: (referralId) => request(`/referral/${referralId}/claim`, { method: 'POST', headers: authHeaders() }),

  getDailyRewardStatus: () => request('/rewards/daily-status', { headers: authHeaders() }),
  claimDailyReward: () => request('/rewards/daily-claim', { method: 'POST', headers: authHeaders() }),
  getAdsStatus: () => request('/rewards/ads-status', { headers: authHeaders() }),

  getLeaderboard: (scope, scopeValue) =>
    request(`/leaderboard?scope=${scope}${scopeValue ? `&scope_value=${scopeValue}` : ''}`, { headers: authHeaders() }),

  getNotifications: () => request('/notifications', { headers: authHeaders() }),
  markNotificationRead: (id) => request(`/notifications/${id}/read`, { method: 'POST', headers: authHeaders() }),
  updateNotificationSettings: (enabled) => request('/notifications/settings', { method: 'PATCH', headers: authHeaders(), body: JSON.stringify({ enabled }) }),
};

export const gameApi = {
  getMatch: (matchId) => request(`/match/${matchId}`, { headers: authHeaders() }),
  searchCustomMatch: (username) => request(`/custom-match/search?username=${encodeURIComponent(username)}`, { headers: authHeaders() }),
  sendCustomMatchInvite: (receiverUsername) =>
    request('/custom-match/invite', { method: 'POST', headers: authHeaders(), body: JSON.stringify({ receiver_username: receiverUsername }) }),
  respondToInvite: (inviteId, decision) =>
    request(`/custom-match/invite/${inviteId}/respond`, { method: 'POST', headers: authHeaders(), body: JSON.stringify({ decision }) }),
  getInviteHistory: () => request('/custom-match/history', { headers: authHeaders() }),
  requestHint: (matchId, paidViaAd) =>
    request(`/match/${matchId}/hint`, { method: 'POST', headers: authHeaders(), body: JSON.stringify({ paid_via_ad: paidViaAd }) }),
};

export const supportApi = {
  submitBugReport: (title, description, screenshotUrl) =>
    request('/reports/bug', { method: 'POST', headers: authHeaders(), body: JSON.stringify({ title, description, screenshot_url: screenshotUrl }) }),
  getSupportInfo: () => request('/support/info'),
  getPrivacyPolicy: () => request('/legal/privacy-policy'),
  getTermsOfService: () => request('/legal/terms-of-service'),
  getAbout: () => request('/legal/about'),
};
