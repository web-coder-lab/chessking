/**
 * Build a shareable PGN string from SAN moves + optional headers.
 */
export function buildPgn({
  movesSan = [],
  whiteName = 'White',
  blackName = 'Black',
  result = '*',
  matchType = 'casual',
  matchId = '',
}) {
  const resultTag =
    result === 'white_win' || result === '1-0'
      ? '1-0'
      : result === 'black_win' || result === '0-1'
        ? '0-1'
        : result === 'draw' || result === '1/2-1/2'
          ? '1/2-1/2'
          : '*';

  const headers = [
    `[Event "Genius Clan ${matchType}"]`,
    `[Site "https://genius-clan.onrender.com"]`,
    `[Date "${new Date().toISOString().slice(0, 10).replace(/-/g, '.')}"]`,
    `[White "${whiteName}"]`,
    `[Black "${blackName}"]`,
    `[Result "${resultTag}"]`,
  ];
  if (matchId) headers.push(`[MatchId "${matchId}"]`);

  let body = '';
  for (let i = 0; i < movesSan.length; i++) {
    if (i % 2 === 0) body += `${Math.floor(i / 2) + 1}. `;
    body += `${movesSan[i]} `;
  }
  body += resultTag;

  return `${headers.join('\n')}\n\n${body.trim()}\n`;
}

export async function copyText(text) {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    try {
      const ta = document.createElement('textarea');
      ta.value = text;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand('copy');
      document.body.removeChild(ta);
      return true;
    } catch {
      return false;
    }
  }
}

export async function sharePgn(pgn, title = 'Genius Clan game') {
  if (navigator.share) {
    try {
      await navigator.share({ title, text: pgn });
      return true;
    } catch {
      return copyText(pgn);
    }
  }
  return copyText(pgn);
}
