package com.geniusclan.app.ui.screens.wallet

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
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.modifier.Modifier
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
fun WalletScreen(
    onBack: () -> Unit,
    onAddCoins: () -> Unit = {},
    onHistory: () -> Unit = {}
) {
    var balance by remember { mutableStateOf<Long?>(null) }
    var loading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()

    fun refresh() {
        loading = true
        error = null
        scope.launch {
            val r = withContext(Dispatchers.IO) { ApiClient.getWalletBalance() }
            loading = false
            r.onSuccess { balance = it.coinBalance }.onFailure { error = it.message }
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
        TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
        Text("Wallet", color = GcText, fontWeight = FontWeight.Bold, fontSize = 26.sp)
        Text("Coins & deposits", color = GcTextMuted, fontSize = 13.sp)
        Spacer(Modifier.height(20.dp))

        Column(
            modifier = Modifier
                .fillMaxWidth()
                .background(GcSurface, RoundedCornerShape(18.dp))
                .border(1.dp, GcBorder, RoundedCornerShape(18.dp))
                .padding(24.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text("Balance", color = GcTextMuted, fontSize = 13.sp)
            if (loading) {
                CircularProgressIndicator(color = GcGold, modifier = Modifier.padding(16.dp))
            } else {
                Text("${balance ?: 0}", color = GcGold, fontWeight = FontWeight.Bold, fontSize = 40.sp)
                Text("coins", color = GcTextMuted, fontSize = 14.sp)
            }
            error?.let { Text(it, color = GcDanger, fontSize = 12.sp, modifier = Modifier.padding(top = 8.dp)) }
        }

        Spacer(Modifier.height(14.dp))
        Button(
            onClick = onAddCoins,
            modifier = Modifier.fillMaxWidth().height(50.dp),
            shape = RoundedCornerShape(12.dp),
            colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
        ) { Text("Add coins", fontWeight = FontWeight.SemiBold) }

        Spacer(Modifier.height(10.dp))
        Button(
            onClick = onHistory,
            modifier = Modifier.fillMaxWidth().height(48.dp),
            shape = RoundedCornerShape(12.dp),
            colors = ButtonDefaults.buttonColors(containerColor = GcSurface, contentColor = GcGold)
        ) { Text("Transaction history") }

        Spacer(Modifier.height(10.dp))
        TextButton(onClick = { refresh() }, modifier = Modifier.align(Alignment.CenterHorizontally)) {
            Text("Refresh balance", color = GcTextMuted)
        }
    }
}
