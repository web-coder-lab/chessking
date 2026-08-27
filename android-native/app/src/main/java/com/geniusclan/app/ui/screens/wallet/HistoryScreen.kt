package com.geniusclan.app.ui.screens.wallet

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
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
fun HistoryScreen(onBack: () -> Unit) {
    var loading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }
    var rows by remember { mutableStateOf(listOf<ApiClient.HistoryRowDto>()) }

    LaunchedEffect(Unit) {
        val r = withContext(Dispatchers.IO) { ApiClient.walletHistory() }
        loading = false
        r.onSuccess { rows = it }.onFailure { error = it.message }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .verticalScroll(rememberScrollState())
            .padding(20.dp)
    ) {
        TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
        Text("History", color = GcText, fontWeight = FontWeight.Bold, fontSize = 24.sp)
        if (loading) CircularProgressIndicator(color = GcGold, modifier = Modifier.padding(24.dp))
        error?.let { Text(it, color = GcDanger) }
        rows.forEach { row ->
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 8.dp)
                    .background(GcSurface, RoundedCornerShape(12.dp))
                    .border(1.dp, GcBorder, RoundedCornerShape(12.dp))
                    .padding(14.dp)
            ) {
                Text(row.label, color = GcText, fontWeight = FontWeight.SemiBold)
                if (row.amount.isNotBlank()) Text(row.amount, color = GcGold, fontSize = 13.sp)
                if (row.createdAt.isNotBlank()) Text(row.createdAt, color = GcTextMuted, fontSize = 11.sp)
            }
        }
        if (!loading && rows.isEmpty() && error == null) {
            Text("No transactions yet.", color = GcTextMuted, modifier = Modifier.padding(top = 12.dp))
        }
    }
}
