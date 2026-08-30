package com.geniusclan.app.ui.screens.hub

import android.content.Intent
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
import androidx.compose.ui.platform.LocalContext
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
fun InviteScreen(onBack: () -> Unit) {
    val context = LocalContext.current
    var loading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }
    var code by remember { mutableStateOf("") }
    var url by remember { mutableStateOf("") }

    LaunchedEffect(Unit) {
        val r = withContext(Dispatchers.IO) { ApiClient.referralLink() }
        loading = false
        r.onSuccess {
            code = it.code
            url = it.shareUrl
        }.onFailure { error = it.message }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .padding(20.dp)
    ) {
        TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
        Text("Invite friend", color = GcText, fontWeight = FontWeight.Bold, fontSize = 24.sp)
        Text("Share your code and earn rewards", color = GcTextMuted, fontSize = 13.sp)
        Spacer(Modifier.height(20.dp))
        if (loading) CircularProgressIndicator(color = GcGold)
        error?.let { Text(it, color = GcDanger) }
        if (!loading && error == null) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(GcSurface, RoundedCornerShape(16.dp))
                    .border(1.dp, GcBorder, RoundedCornerShape(16.dp))
                    .padding(20.dp)
            ) {
                Text("Your code", color = GcTextMuted, fontSize = 12.sp)
                Text(code.ifBlank { "—" }, color = GcGold, fontWeight = FontWeight.Bold, fontSize = 28.sp)
                if (url.isNotBlank()) {
                    Text(url, color = GcTextMuted, fontSize = 12.sp, modifier = Modifier.padding(top = 8.dp))
                }
            }
            Spacer(Modifier.height(16.dp))
            Button(
                onClick = {
                    val share = Intent(Intent.ACTION_SEND).apply {
                        type = "text/plain"
                        putExtra(
                            Intent.EXTRA_TEXT,
                            "Join Genius Clan! Code: $code\n$url"
                        )
                    }
                    context.startActivity(Intent.createChooser(share, "Invite friend"))
                },
                modifier = Modifier.fillMaxWidth().height(50.dp),
                shape = RoundedCornerShape(12.dp),
                colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
            ) { Text("Share invite", fontWeight = FontWeight.SemiBold) }
        }
    }
}
