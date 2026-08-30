package com.geniusclan.app.ui.screens.board

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.geniusclan.app.chess.Fen
import com.geniusclan.app.data.ws.GameSocket
import com.geniusclan.app.ui.theme.GcBg
import com.geniusclan.app.ui.theme.GcBoardDark
import com.geniusclan.app.ui.theme.GcBoardLight
import com.geniusclan.app.ui.theme.GcDanger
import com.geniusclan.app.ui.theme.GcGold
import com.geniusclan.app.ui.theme.GcGoldSoft
import com.geniusclan.app.ui.theme.GcText
import com.geniusclan.app.ui.theme.GcTextMuted
import org.json.JSONObject

@Composable
fun BoardScreen(
    matchId: String,
    myColor: String,
    onLeave: () -> Unit
) {
    var fen by remember { mutableStateOf(Fen.START) }
    var status by remember { mutableStateOf("Connecting…") }
    var selected by remember { mutableStateOf<Pair<Int, Int>?>(null) }
    var ended by remember { mutableStateOf<String?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    var showLeaveConfirm by remember { mutableStateOf(false) }
    var socket by remember { mutableStateOf<GameSocket?>(null) }

    val iAmWhite = myColor.equals("white", true)
    val board = remember(fen) { Fen.board(fen) }

    DisposableEffect(matchId) {
        val s = GameSocket(
            onEvent = { msg ->
                when (msg.optString("type")) {
                    "board_sync", "board_update" -> {
                        val f = msg.optString("fen")
                        if (f.isNotBlank()) fen = f
                        status = "Playing"
                    }
                    "match_found" -> {
                        val f = msg.optString("fen")
                        if (f.isNotBlank()) fen = f
                        status = "Match ready"
                    }
                    "match_ended" -> {
                        ended = msg.optString("result", msg.optString("result_reason", "ended"))
                        status = "Game over"
                    }
                    "draw_offered" -> {
                        status = "Draw offered — accept or decline"
                    }
                    "draw_declined" -> status = "Draw declined"
                    "error" -> error = msg.optString("message", msg.optString("code"))
                    "opponent_disconnected" -> status = "Opponent disconnected"
                    "opponent_reconnected" -> status = "Opponent back"
                }
            },
            onState = { st ->
                status = st
                if (st == "open") {
                    s.resumeMatch(matchId)
                }
            }
        )
        socket = s
        s.connectMatch(matchId)
        onDispose { s.close() }
    }

    fun tryMove(r: Int, c: Int) {
        if (ended != null) return
        val sel = selected
        if (sel == null) {
            val piece = board[r][c] ?: return
            val whitePiece = Fen.isWhitePiece(piece)
            if (whitePiece != iAmWhite) return
            selected = r to c
            return
        }
        if (sel.first == r && sel.second == c) {
            selected = null
            return
        }
        val from = Fen.square(sel.first, sel.second)
        val to = Fen.square(r, c)
        selected = null
        error = null
        // Auto-queen if pawn to last rank
        val piece = board[sel.first][sel.second]
        val promo = when {
            piece == 'P' && r == 0 -> "q"
            piece == 'p' && r == 7 -> "q"
            else -> null
        }
        socket?.move(matchId, from, to, promo)
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .padding(12.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            TextButton(onClick = { showLeaveConfirm = true }) { Text("← Leave", color = GcGold) }
            Text("You: $myColor", color = GcTextMuted, fontSize = 13.sp)
        }
        Text(status, color = GcText, fontWeight = FontWeight.SemiBold)
        ended?.let { Text("Result: $it", color = GcGold, fontSize = 14.sp) }
        error?.let { Text(it, color = GcDanger, fontSize = 13.sp) }

        Spacer(Modifier.height(8.dp))

        BoxWithConstraints(
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(1f)
        ) {
            val cell = maxWidth / 8
            Column {
                for (r in 0 until 8) {
                    Row {
                        for (c in 0 until 8) {
                            val light = (r + c) % 2 == 0
                            val isSel = selected?.first == r && selected?.second == c
                            Box(
                                modifier = Modifier
                                    .size(cell)
                                    .background(if (light) GcBoardLight else GcBoardDark)
                                    .then(
                                        if (isSel) Modifier.border(2.dp, GcGold)
                                        else Modifier
                                    )
                                    .clickable { tryMove(r, c) },
                                contentAlignment = Alignment.Center
                            ) {
                                Text(
                                    text = Fen.glyph(board[r][c]),
                                    fontSize = 28.sp,
                                    color = if (Fen.isWhitePiece(board[r][c])) GcText else GcBg
                                )
                            }
                        }
                    }
                }
            }
        }

        Spacer(Modifier.height(12.dp))
        if (status.contains("Draw offered", ignoreCase = true)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Button(
                    onClick = { socket?.acceptDraw(matchId) },
                    modifier = Modifier.weight(1f),
                    colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
                ) { Text("Accept draw") }
                Button(
                    onClick = { socket?.declineDraw(matchId) },
                    modifier = Modifier.weight(1f),
                    colors = ButtonDefaults.buttonColors(containerColor = GcSurface, contentColor = GcText)
                ) { Text("Decline") }
            }
            Spacer(Modifier.height(8.dp))
        }
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Button(
                onClick = { socket?.offerDraw(matchId) },
                modifier = Modifier.weight(1f),
                colors = ButtonDefaults.buttonColors(containerColor = GcGoldSoft, contentColor = GcGold)
            ) { Text("Offer draw") }
            Button(
                onClick = { socket?.resign(matchId) },
                modifier = Modifier.weight(1f),
                colors = ButtonDefaults.buttonColors(containerColor = GcDanger, contentColor = GcText)
            ) { Text("Resign") }
        }
        if (ended != null) {
            Spacer(Modifier.height(12.dp))
            Text("Match finished", color = GcGold, fontWeight = FontWeight.Bold, fontSize = 18.sp)
            Button(
                onClick = onLeave,
                modifier = Modifier.fillMaxWidth().padding(top = 8.dp),
                colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
            ) { Text("Back to home") }
        }
    }

    if (showLeaveConfirm) {
        AlertDialog(
            onDismissRequest = { showLeaveConfirm = false },
            title = { Text("Leave match?") },
            text = { Text("Do you want to leave this match? You may resign or lose the game.") },
            confirmButton = {
                TextButton(onClick = { showLeaveConfirm = false; onLeave() }) {
                    Text("Leave", color = GcDanger)
                }
            },
            dismissButton = {
                TextButton(onClick = { showLeaveConfirm = false }) {
                    Text("Cancel", color = GcGold)
                }
            }
        )
    }
}
