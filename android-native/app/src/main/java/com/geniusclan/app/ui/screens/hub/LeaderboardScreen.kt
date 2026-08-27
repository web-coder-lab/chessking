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
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier.Modifier
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
import kotlinx.coroutines.withContext

@Composable
fun LeaderboardScreen(onBack: () -> Unit) {
    var loading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }
    var rows by remember { mutableStateOf(listOf<ApiClient.LeaderboardEntryDto>()) }
    var myRank by remember { mutableStateOf<Long?>(null) }

    LaunchedEffect(Unit) {
        val r = withContext(Dispatchers.IO) { ApiClient.leaderboard("global") }
        loading = false
        r.onSuccess {
            rows = it.first
            myRank = it.second
        }.onFailure { error = it.message }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .verticalScroll(rememberScrollState())
            .padding(20.dp)
    ) {
        TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
        Text("Leaderboard", color = GcText, fontWeight = FontWeight.Bold, fontSize = 24.sp)
        myRank?.let {
            Text("Your rank: #$it", color = GcGold, fontSize = 14.sp, modifier = Modifier.padding(top = 4.dp))
        }
        Spacer(Modifier.height(12.dp))
        if (loading) CircularProgressIndicator(color = GcGold)
        error?.let { Text(it, color = GcDanger) }
        rows.forEach { row ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 8.dp)
                    .background(GcSurface, RoundedCornerShape(12.dp))
                    .border(1.dp, GcBorder, RoundedCornerShape(12.dp))
                    .padding(14.dp)
            ) {
                Text("#${row.rank}", color = GcGold, fontWeight = FontWeight.Bold, modifier = Modifier.padding(end = 12.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(row.username, color = GcText, fontWeight = FontWeight.SemiBold)
                    Text("Rating ${row.rating}", color = GcTextMuted, fontSize = 12.sp)
                }
            }
        }
        if (!loading && rows.isEmpty() && error == null) {
            Text("No rankings yet.", color = GcTextMuted)
        }
    }
}
