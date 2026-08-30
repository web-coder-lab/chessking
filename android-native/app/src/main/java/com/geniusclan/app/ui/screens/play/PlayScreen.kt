package com.geniusclan.app.ui.screens.play

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
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
import com.geniusclan.app.data.ws.GameSocket
import com.geniusclan.app.ui.theme.GcBg
import com.geniusclan.app.ui.theme.GcBorder
import com.geniusclan.app.ui.theme.GcDanger
import com.geniusclan.app.ui.theme.GcGold
import com.geniusclan.app.ui.theme.GcSurface
import com.geniusclan.app.ui.theme.GcText
import com.geniusclan.app.ui.theme.GcTextMuted
import org.json.JSONObject

@Composable
fun PlayScreen(
    onBack: () -> Unit,
    onMatchFound: (matchId: String, color: String) -> Unit
) {
    var status by remember { mutableStateOf("Idle") }
    var searching by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var socket by remember { mutableStateOf<GameSocket?>(null) }

    fun stop() {
        searching = false
        socket?.close()
        socket = null
        status = "Idle"
    }

    fun startQueue(type: String) {
        error = null
        searching = true
        status = "Connecting…"
        val s = GameSocket(
            onEvent = { msg ->
                val t = msg.optString("type")
                when (t) {
                    "match_found" -> {
                        val id = msg.optString("match_id")
                        val color = msg.optString("color", msg.optString("your_color", "white"))
                        searching = false
                        status = "Match found"
                        onMatchFound(id, color)
                    }
                    "error" -> {
                        error = msg.optString("message", msg.optString("code", "error"))
                        searching = false
                    }
                    else -> status = t.ifBlank { "event" }
                }
            },
            onState = { st ->
                status = st
                if (st.startsWith("error:")) {
                    error = st.removePrefix("error:")
                    searching = false
                }
                if (st == "open") {
                    socket?.joinQueue(type)
                    status = "Searching ($type)…"
                }
            }
        )
        socket = s
        s.connectQueue()
    }

    DisposableEffect(Unit) {
        onDispose { stop() }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .padding(20.dp)
    ) {
        TextButton(onClick = { stop(); onBack() }) { Text("← Back", color = GcGold) }
        Text("Play", color = GcText, fontWeight = FontWeight.Bold, fontSize = 28.sp)
        Text("Real matchmaking via WebSocket", color = GcTextMuted, fontSize = 13.sp)

        Spacer(Modifier.height(24.dp))
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .background(GcSurface, RoundedCornerShape(16.dp))
                .border(1.dp, GcBorder, RoundedCornerShape(16.dp))
                .padding(20.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text("♚", fontSize = 48.sp, color = GcGold)
            Text(status, color = GcText, fontWeight = FontWeight.SemiBold, fontSize = 16.sp)
            error?.let { Text(it, color = GcDanger, fontSize = 13.sp, modifier = Modifier.padding(top = 8.dp)) }
        }

        Spacer(Modifier.height(20.dp))
        if (!searching) {
            Button(
                onClick = { startQueue("casual") },
                modifier = Modifier.fillMaxWidth().height(52.dp),
                shape = RoundedCornerShape(14.dp),
                colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
            ) { Text("Casual match", fontWeight = FontWeight.SemiBold) }
            Spacer(Modifier.height(10.dp))
            Button(
                onClick = { startQueue("ranked") },
                modifier = Modifier.fillMaxWidth().height(52.dp),
                shape = RoundedCornerShape(14.dp),
                colors = ButtonDefaults.buttonColors(containerColor = GcSurface, contentColor = GcGold)
            ) { Text("Ranked match", fontWeight = FontWeight.SemiBold) }
        } else {
            Button(
                onClick = { stop() },
                modifier = Modifier.fillMaxWidth().height(52.dp),
                shape = RoundedCornerShape(14.dp),
                colors = ButtonDefaults.buttonColors(containerColor = GcDanger, contentColor = GcText)
            ) { Text("Cancel search") }
        }

        Spacer(Modifier.height(16.dp))
        Text(
            "Need two logged-in devices/accounts in queue to pair.\nServer validates all moves.",
            color = GcTextMuted,
            fontSize = 12.sp,
            lineHeight = 18.sp
        )
    }
}
