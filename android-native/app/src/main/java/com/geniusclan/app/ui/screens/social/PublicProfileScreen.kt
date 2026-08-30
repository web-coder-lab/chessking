package com.geniusclan.app.ui.screens.social

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
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
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
fun PublicProfileScreen(onBack: () -> Unit, onSendGift: (String) -> Unit) {
    var query by remember { mutableStateOf("") }
    var loading by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var profile by remember { mutableStateOf<ApiClient.PublicProfileDto?>(null) }
    val scope = rememberCoroutineScope()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .padding(20.dp)
    ) {
        TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
        Text("Find player", color = GcText, fontWeight = FontWeight.Bold, fontSize = 24.sp)
        Spacer(Modifier.height(12.dp))
        OutlinedTextField(
            value = query,
            onValueChange = { query = it },
            label = { Text("Username") },
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
        Spacer(Modifier.height(8.dp))
        Button(
            onClick = {
                loading = true
                error = null
                profile = null
                scope.launch {
                    val r = withContext(Dispatchers.IO) { ApiClient.getPublicProfile(query.trim()) }
                    loading = false
                    r.onSuccess { profile = it }.onFailure { error = it.message }
                }
            },
            enabled = query.isNotBlank() && !loading,
            modifier = Modifier.fillMaxWidth(),
            shape = RoundedCornerShape(12.dp),
            colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
        ) { Text(if (loading) "Loading…" else "View profile") }

        error?.let { Text(it, color = GcDanger, modifier = Modifier.padding(top = 8.dp)) }
        if (loading) CircularProgressIndicator(color = GcGold, modifier = Modifier.padding(16.dp))

        profile?.let { p ->
            Spacer(Modifier.height(16.dp))
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(GcSurface, RoundedCornerShape(16.dp))
                    .border(1.dp, GcBorder, RoundedCornerShape(16.dp))
                    .padding(18.dp)
            ) {
                Text(p.username, color = GcText, fontWeight = FontWeight.Bold, fontSize = 22.sp)
                Text("Rating ${p.rating}", color = GcGold, fontSize = 14.sp)
                Text("W/L ${p.wins}/${p.losses}", color = GcTextMuted, fontSize = 13.sp)
                p.bio?.let {
                    Text(it, color = GcTextMuted, fontSize = 13.sp, modifier = Modifier.padding(top = 8.dp))
                }
                Spacer(Modifier.height(12.dp))
                Button(
                    onClick = { onSendGift(p.username) },
                    shape = RoundedCornerShape(10.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
                ) { Text("Send gift") }
            }
        }
    }
}
