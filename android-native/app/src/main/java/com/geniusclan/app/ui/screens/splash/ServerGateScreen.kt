package com.geniusclan.app.ui.screens.splash

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.geniusclan.app.BuildConfig
import com.geniusclan.app.ui.theme.GcBg
import com.geniusclan.app.ui.theme.GcGold
import com.geniusclan.app.ui.theme.GcTextMuted
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import okhttp3.OkHttpClient
import okhttp3.Request
import java.util.concurrent.TimeUnit

@Composable
fun ServerGateScreen(onReady: () -> Unit) {
    var offline by remember { mutableStateOf(false) }
    var showHint by remember { mutableStateOf(false) }

    LaunchedEffect(Unit) {
        val client = OkHttpClient.Builder()
            .connectTimeout(8, TimeUnit.SECONDS)
            .readTimeout(8, TimeUnit.SECONDS)
            .build()
        val url = "${BuildConfig.API_BASE_URL}/health"
        // First quick check — if online, skip long UI
        if (ping(client, url)) {
            onReady()
            return@LaunchedEffect
        }
        offline = true
        delay(2500)
        showHint = true
        while (isActive) {
            if (ping(client, url)) {
                onReady()
                return@LaunchedEffect
            }
            delay(4000)
        }
    }

    val pulse = rememberInfiniteTransition(label = "pulse")
    val scale by pulse.animateFloat(
        initialValue = 0.85f,
        targetValue = 1.2f,
        animationSpec = infiniteRepeatable(
            animation = tween(1400, easing = LinearEasing),
            repeatMode = RepeatMode.Restart
        ),
        label = "scale"
    )

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg),
        contentAlignment = Alignment.Center
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(16.dp),
            modifier = Modifier.padding(24.dp)
        ) {
            Box(contentAlignment = Alignment.Center) {
                Box(
                    modifier = Modifier
                        .size(88.dp)
                        .scale(scale)
                        .background(GcGold.copy(alpha = 0.15f), CircleShape)
                )
                Text(text = "♚", fontSize = 40.sp, color = GcGold)
            }
            Text(
                text = if (offline) "Connecting server…" else "Checking server…",
                color = Color.White,
                fontWeight = FontWeight.SemiBold,
                fontSize = 20.sp
            )
            if (showHint) {
                Text(
                    text = "Please wait 30–60 seconds\nFree server may be waking up",
                    color = GcGold,
                    fontSize = 14.sp,
                    textAlign = TextAlign.Center,
                    lineHeight = 20.sp
                )
            }
            Text(text = "API ${BuildConfig.API_BASE_URL}", color = GcTextMuted, fontSize = 11.sp)
        }
    }
}

private fun ping(client: OkHttpClient, url: String): Boolean {
    return try {
        val req = Request.Builder().url(url).get().build()
        client.newCall(req).execute().use { it.isSuccessful }
    } catch (_: Exception) {
        false
    }
}
