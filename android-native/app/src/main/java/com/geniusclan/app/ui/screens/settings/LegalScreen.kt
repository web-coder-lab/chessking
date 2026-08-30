package com.geniusclan.app.ui.screens.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
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
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.geniusclan.app.data.api.ApiClient
import com.geniusclan.app.ui.theme.GcBg
import com.geniusclan.app.ui.theme.GcDanger
import com.geniusclan.app.ui.theme.GcGold
import com.geniusclan.app.ui.theme.GcText
import com.geniusclan.app.ui.theme.GcTextMuted
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

enum class LegalKind { PRIVACY, TERMS, ABOUT, SUPPORT }

@Composable
fun LegalScreen(kind: LegalKind, onBack: () -> Unit) {
    var loading by remember { mutableStateOf(true) }
    var body by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    val title = when (kind) {
        LegalKind.PRIVACY -> "Privacy policy"
        LegalKind.TERMS -> "Terms of service"
        LegalKind.ABOUT -> "About"
        LegalKind.SUPPORT -> "Support"
    }

    LaunchedEffect(kind) {
        val r = withContext(Dispatchers.IO) {
            when (kind) {
                LegalKind.PRIVACY -> ApiClient.fetchLegal("/legal/privacy-policy")
                LegalKind.TERMS -> ApiClient.fetchLegal("/legal/terms-of-service")
                LegalKind.ABOUT -> ApiClient.fetchLegal("/legal/about")
                LegalKind.SUPPORT -> ApiClient.supportInfo()
            }
        }
        loading = false
        r.onSuccess { body = it }.onFailure { error = it.message }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .verticalScroll(rememberScrollState())
            .padding(20.dp)
    ) {
        TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
        Text(title, color = GcText, fontWeight = FontWeight.Bold, fontSize = 22.sp)
        if (loading) CircularProgressIndicator(color = GcGold, modifier = Modifier.padding(24.dp))
        error?.let { Text(it, color = GcDanger) }
        if (!loading && error == null) {
            Text(
                text = body.ifBlank { "Content not available yet." },
                color = GcTextMuted,
                fontSize = 14.sp,
                lineHeight = 22.sp,
                modifier = Modifier.padding(top = 12.dp)
            )
        }
    }
}
