package com.geniusclan.app.ui.screens.settings

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
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
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
fun SessionsScreen(onBack: () -> Unit) {
    var loading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }
    var sessions by remember { mutableStateOf(listOf<ApiClient.SessionDto>()) }
    val scope = rememberCoroutineScope()

    fun reload() {
        loading = true
        scope.launch {
            val r = withContext(Dispatchers.IO) { ApiClient.getSessions() }
            loading = false
            r.onSuccess { sessions = it }.onFailure { error = it.message }
        }
    }

    LaunchedEffect(Unit) { reload() }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .verticalScroll(rememberScrollState())
            .padding(20.dp)
    ) {
        TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
        Text("Active sessions", color = GcText, fontWeight = FontWeight.Bold, fontSize = 22.sp)
        Spacer(Modifier.height(12.dp))
        if (loading) {
            CircularProgressIndicator(color = GcGold, modifier = Modifier.align(Alignment.CenterHorizontally))
        }
        error?.let { Text(it, color = GcDanger) }
        if (!loading && sessions.isEmpty() && error == null) {
            Text("No sessions returned.", color = GcTextMuted)
        }
        sessions.forEach { s ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 8.dp)
                    .background(GcSurface, RoundedCornerShape(12.dp))
                    .border(1.dp, GcBorder, RoundedCornerShape(12.dp))
                    .padding(12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column(Modifier.weight(1f)) {
                    Text(s.label, color = GcText, fontWeight = FontWeight.SemiBold)
                    if (s.current) Text("This device", color = GcGold, fontSize = 12.sp)
                }
                if (!s.current && s.id.isNotBlank()) {
                    TextButton(onClick = {
                        scope.launch {
                            withContext(Dispatchers.IO) { ApiClient.revokeSession(s.id) }
                            reload()
                        }
                    }) { Text("Revoke", color = GcDanger) }
                }
            }
        }
    }
}
