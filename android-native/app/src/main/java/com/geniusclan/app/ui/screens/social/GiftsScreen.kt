package com.geniusclan.app.ui.screens.social

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
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier.modifier
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
fun GiftsScreen(onBack: () -> Unit, prefillUsername: String = "") {
    var receiver by remember { mutableStateOf(prefillUsername) }
    var items by remember { mutableStateOf(listOf<ApiClient.GiftItemDto>()) }
    var loading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }
    var message by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        val r = withContext(Dispatchers.IO) { ApiClient.giftCatalog() }
        loading = false
        r.onSuccess { items = it }.onFailure { error = it.message }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .verticalScroll(rememberScrollState())
            .padding(20.dp)
    ) {
        TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
        Text("Send gift", color = GcText, fontWeight = FontWeight.Bold, fontSize = 24.sp)
        Spacer(Modifier.height(12.dp))
        OutlinedTextField(
            value = receiver,
            onValueChange = { receiver = it },
            label = { Text("Receiver username") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
            shape = RoundedCornerShape(12.dp),
            colors = OutlinedTextFieldDefaults.colors(
                focusedBorderColor = GcGold,
                unfocusedBorderColor = GcBorder,
                focusedContainerColor = GcSurface,
                unfocusedContainerColor = GcSurface,
                focusedTextColor = GcText,
                unfocusedTextColor = GcText,
                cursorColor = GcGold
            )
        )
        Spacer(Modifier.height(12.dp))
        if (loading) CircularProgressIndicator(color = GcGold)
        error?.let { Text(it, color = GcDanger) }
        message?.let { Text(it, color = GcGold) }
        items.forEach { g ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 8.dp)
                    .background(GcSurface, RoundedCornerShape(12.dp))
                    .border(1.dp, GcBorder, RoundedCornerShape(12.dp))
                    .padding(12.dp)
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(g.name, color = GcText, fontWeight = FontWeight.SemiBold)
                    Text("${g.priceCoins} coins", color = GcGold, fontSize = 12.sp)
                }
                Button(
                    onClick = {
                        if (receiver.isBlank()) {
                            error = "Enter receiver username"
                            return@Button
                        }
                        scope.launch {
                            val r = withContext(Dispatchers.IO) {
                                ApiClient.sendGift(receiver.trim(), g.id)
                            }
                            r.onSuccess { message = "Gift sent to $receiver" }
                                .onFailure { error = it.message }
                        }
                    },
                    shape = RoundedCornerShape(10.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
                ) { Text("Send") }
            }
        }
        if (!loading && items.isEmpty() && error == null) {
            Text("No gifts in catalog.", color = GcTextMuted)
        }
    }
}
