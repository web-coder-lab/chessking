import './Board.css';

const PIECE_UNICODE = {
  p: '♟', r: '♜', n: '♞', b: '♝', q: '♛', k: '♚',
  P: '♙', R: '♖', N: '♘', B: '♗', Q: '♕', K: '♔',
};

const FILES = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

/**
 * Visual board — state is always driven by parent (server-backed FEN).
 */
export default function Board({
  fen,
  orientation = 'white',
  onMove,
  selectedSquare,
  legalTargets = [],
  lastMove,
  hintMove,
  checkSquare = null,
}) {
  const rows = parseFen(fen);
  const displayRows =
    orientation === 'white' ? rows : [...rows].reverse().map((r) => [...r].reverse());

  return (
    <div className="ck-board" role="grid" aria-label="Chess board">
      {displayRows.map((row, rIdx) => (
        <div className="ck-board__row" key={rIdx} role="row">
          {row.map((piece, cIdx) => {
            const fileIdx = orientation === 'white' ? cIdx : 7 - cIdx;
            const rankIdx = orientation === 'white' ? rIdx : 7 - rIdx;
            const square = `${FILES[fileIdx]}${8 - rankIdx}`;
            const isLight = (fileIdx + rankIdx) % 2 === 0;
            const isSelected = selectedSquare === square;
            const isTarget = legalTargets.includes(square);
            const isLastMove = lastMove && (lastMove.from === square || lastMove.to === square);
            const isHint = hintMove && (hintMove.from === square || hintMove.to === square);
            const isCheck = checkSquare === square;

            return (
              <button
                key={square}
                type="button"
                className={[
                  'ck-board__square',
                  isLight ? 'ck-board__square--light' : 'ck-board__square--dark',
                  isSelected ? 'ck-board__square--selected' : '',
                  isLastMove ? 'ck-board__square--last-move' : '',
                  isHint ? 'ck-board__square--hint' : '',
                  isCheck ? 'ck-board__square--check' : '',
                ]
                  .join(' ')
                  .trim()}
                onClick={() => onMove(square)}
                aria-label={square}
              >
                {piece && (
                  <span
                    className={`ck-board__piece ${
                      piece === piece.toUpperCase()
                        ? 'ck-board__piece--white'
                        : 'ck-board__piece--black'
                    }`}
                  >
                    {PIECE_UNICODE[piece]}
                  </span>
                )}
                {isTarget && <span className="ck-board__target-dot" aria-hidden="true" />}
              </button>
            );
          })}
        </div>
      ))}
    </div>
  );
}

function parseFen(fen) {
  if (!fen) return Array.from({ length: 8 }, () => Array(8).fill(null));
  const boardPart = fen.split(' ')[0];
  return boardPart.split('/').map((rowStr) => {
    const row = [];
    for (const char of rowStr) {
      if (/\d/.test(char)) {
        for (let i = 0; i < Number(char); i++) row.push(null);
      } else {
        row.push(char);
      }
    }
    return row;
  });
}
