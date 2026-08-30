package com.geniusclan.app.ui.screens.hub

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.geniusclan.app.data.api.ApiClient
import com.geniusclan.app.ui.theme.GcBg
import com.geniusclan.app.ui.theme.GcBorder
import com.geniusclan.app.ui.theme.GcDanger
import com.geniusclan.app.ui.theme.GcGold
import com.geniusclan.app.ui.theme.GcSurface
import com.geniusclan.app.ui.theme.GcText
import com.geniusclan.app.ui.theme.GcTextMuted
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@Composable
fun DashboardScreen(
    onBack: () -> Unit,
    onPlay: () -> Unit,
    onLeaderboard: () -> Unit,
    onNotifications: () -> Unit,
    onInvite: () -> Unit
) {
    var streak by remember { mutableStateOf(0L) }
    var nextCoins by remember { mutableStateOf(0L) }
    var claimed by remember { mutableStateOf(false) }
    var balance by remember { mutableStateOf(0L) }
    var username by remember { mutableStateOf("Player") }
    var rating by remember { mutableStateOf(1200) }
    var message by remember { mutableStateOf<String?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    var claiming by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    fun refresh() {
        scope.launch {
            withContext(Dispatchers.IO) {
                ApiClient.dailyStatus().onSuccess {
                    claimed = it.claimedToday
                    streak = it.streakDay
                    nextCoins = it.nextCoins
                }
                ApiClient.getWalletBalance().onSuccess { balance = it.coinBalance }
                ApiClient.getMyProfile().onSuccess {
                    username = it.username
                    rating = it.rating
                    balance = it.coinBalance
                }
            }
        }
    }

    LaunchedEffect(Unit) { refresh() }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .verticalScroll(rememberScrollState())
            .padding(20.dp)
    ) {
        TextButton(onClick = onBack) { Text("← Home", color = GcGold) }
        Text("Dashboard", color = GcText, fontWeight = FontWeight.Bold, fontSize = 26.sp)
        Text("Hello, $username", color = GcTextMuted, fontSize = 14.sp)
        Spacer(Modifier.height(16.dp))

        Row(modifier = Modifier.fillMaxWidth()) {
            StatCard("Rating", "$rating", Modifier.weight(1f).padding(end = 6.dp))
            StatCard("Coins", "$balance", Modifier.weight(1f).padding(start = 6.dp))
        }
        Spacer(Modifier.height(10.dp))
        StatCard("Streak day", "$streak", Modifier.fillMaxWidth())

        Spacer(Modifier.height(16.dp))
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .background(GcSurface, RoundedCornerShape(16.dp))
                .border(1.dp, GcBorder, RoundedCornerShape(16.dp))
                .padding(16.dp)
        ) {
            Text("Daily reward", color = GcGold, fontWeight = FontWeight.SemiBold)
            Text(
                if (claimed) "Already claimed today"
                else "Claim $nextCoins coins",
                color = GcTextMuted,
                fontSize = 13.sp
            )
            Spacer(Modifier.height(10.dp))
            Button(
                onClick = {
                    claiming = true
                    error = null
                    scope.launch {
                        val r = withContext(Dispatchers.IO) { ApiClient.claimDaily() }
                        claiming = false
                        r.onSuccess {
                            message = "+$it coins claimed"
                            refresh()
                        }.onFailure { error = it.message }
                    }
                },
                enabled = !claimed && !claiming,
                modifier = Modifier.fillMaxWidth(),
                shape = RoundedCornerShape(12.dp),
                colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
            ) {
                Text(
                    when {
                        claiming -> "Claiming…"
                        claimed -> "Claimed"
                        else -> "Claim daily"
                    }
                )
            }
        }

        message?.let { Text(it, color = GcGold, modifier = Modifier.padding(top = 8.dp)) }
        error?.let { Text(it, color = GcDanger, modifier = Modifier.padding(top = 8.dp)) }

        Spacer(Modifier.height(16.dp))
        Quick("Play now", onPlay)
        Quick("Leaderboard", onLeaderboard)
        Quick("Notifications", onNotifications)
        Quick("Invite friend", onInvite)
    }
}

@Composable
private fun StatCard(label: String, value: String, modifier: Modifier) {
    Column(
        modifier = modifier
            .background(GcSurface, RoundedCornerShape(14.dp))
            .border(1.dp, GcBorder, RoundedCornerShape(14.dp))
            .padding(14.dp)
    ) {
        Text(label, color = GcTextMuted, fontSize = 12.sp)
        Text(value, color = GcGold, fontWeight = FontWeight.Bold, fontSize = 20.sp)
    }
}

@Composable
private fun Quick(title: String, onClick: () -> Unit) {
    Button(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth().padding(bottom = 8.dp).height(48.dp),
        shape = RoundedCornerShape(12.dp),
        colors = ButtonDefaults.buttonColors(containerColor = GcSurface, contentColor = GcText)
    ) { Text(title) }
}
