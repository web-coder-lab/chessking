package com.geniusclan.app.ui.screens.hub

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
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
fun NotificationsScreen(onBack: () -> Unit) {
    var loading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }
    var items by remember { mutableStateOf(listOf<ApiClient.NotificationDto>()) }
    val scope = rememberCoroutineScope()

    fun load() {
        scope.launch {
            loading = true
            val r = withContext(Dispatchers.IO) { ApiClient.notifications() }
            loading = false
            r.onSuccess { items = it }.onFailure { error = it.message }
        }
    }

    LaunchedEffect(Unit) { load() }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .verticalScroll(rememberScrollState())
            .padding(20.dp)
    ) {
        TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
        Text("Notifications", color = GcText, fontWeight = FontWeight.Bold, fontSize = 24.sp)
        if (loading) CircularProgressIndicator(color = GcGold, modifier = Modifier.padding(16.dp))
        error?.let { Text(it, color = GcDanger) }
        items.forEach { n ->
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 8.dp)
                    .background(GcSurface, RoundedCornerShape(12.dp))
                    .border(1.dp, if (n.read) GcBorder else GcGold, RoundedCornerShape(12.dp))
                    .clickable {
                        if (!n.read && n.id.isNotBlank()) {
                            scope.launch {
                                withContext(Dispatchers.IO) { ApiClient.markNotificationRead(n.id) }
                                load()
                            }
                        }
                    }
                    .padding(14.dp)
            ) {
                Text(n.title, color = GcText, fontWeight = FontWeight.SemiBold)
                if (n.body.isNotBlank()) Text(n.body, color = GcTextMuted, fontSize = 13.sp)
                if (!n.read) Text("Tap to mark read", color = GcGold, fontSize = 11.sp)
            }
        }
        if (!loading && items.isEmpty() && error == null) {
            Text("No notifications.", color = GcTextMuted, modifier = Modifier.padding(top = 12.dp))
        }
    }
}
