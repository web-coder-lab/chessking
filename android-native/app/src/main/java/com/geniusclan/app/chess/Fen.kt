package com.geniusclan.app.chess

/** Minimal FEN board for UI — server remains authority for legality. */
object Fen {
    const val START =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

    private val glyphs = mapOf(
        'K' to "♔", 'Q' to "♕", 'R' to "♖", 'B' to "♗", 'N' to "♘", 'P' to "♙",
        'k' to "♚", 'q' to "♛", 'r' to "♜", 'b' to "♝", 'n' to "♞", 'p' to "♟"
    )

    /** row 0 = rank 8, row 7 = rank 1; col 0 = file a */
    fun board(fen: String): Array<Array<Char?>> {
        val placement = fen.trim().split(' ').firstOrNull() ?: START.split(' ')[0]
        val rows = Array(8) { arrayOfNulls<Char?>(8) }
        val ranks = placement.split('/')
        for (r in 0 until 8) {
            val rank = ranks.getOrNull(r) ?: continue
            var c = 0
            for (ch in rank) {
                if (ch.isDigit()) {
                    c += ch - '0'
                } else if (c < 8) {
                    rows[r][c] = ch
                    c++
                }
            }
        }
        return rows
    }

    fun glyph(piece: Char?): String = piece?.let { glyphs[it] } ?: ""

    fun square(row: Int, col: Int): String {
        val file = ('a' + col)
        val rank = (8 - row)
        return "$file$rank"
    }

    fun isWhitePiece(ch: Char?) = ch != null && ch.isUpperCase()
}
