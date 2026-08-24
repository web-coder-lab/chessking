package com.geniusclan.app.ui.screens.profile

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.geniusclan.app.data.api.ApiClient
import com.geniusclan.app.data.api.ProfileDto
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
fun ProfileScreen(onBack: () -> Unit, onLogout: () -> Unit) {
    var profile by remember { mutableStateOf<ProfileDto?>(null) }
    var bio by remember { mutableStateOf("") }
    var loading by remember { mutableStateOf(true) }
    var saving by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var saved by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        val r = withContext(Dispatchers.IO) { ApiClient.getMyProfile() }
        loading = false
        r.onSuccess {
            profile = it
            bio = it.bio.orEmpty()
        }.onFailure { error = it.message }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .padding(20.dp)
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
            Text("Profile", color = GcText, fontWeight = FontWeight.Bold, fontSize = 22.sp)
        }

        if (loading) {
            CircularProgressIndicator(color = GcGold, modifier = Modifier.padding(32.dp).align(Alignment.CenterHorizontally))
            return
        }

        error?.let { Text(it, color = GcDanger, modifier = Modifier.padding(vertical = 8.dp)) }

        profile?.let { p ->
            Text(text = "♚", fontSize = 40.sp, color = GcGold)
            Text(p.username, color = GcText, fontWeight = FontWeight.Bold, fontSize = 24.sp)
            p.email?.let { Text(it, color = GcTextMuted, fontSize = 13.sp) }

            Spacer(Modifier.height(16.dp))
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceEvenly
            ) {
                Stat("Rating", "${p.rating}")
                Stat("Coins", "${p.coinBalance}")
                Stat("W/L", "${p.wins}/${p.losses}")
            }

            Spacer(Modifier.height(20.dp))
            Text("Bio", color = GcTextMuted, fontSize = 13.sp)
            OutlinedTextField(
                value = bio,
                onValueChange = { bio = it.take(300); saved = false },
                modifier = Modifier.fillMaxWidth().height(120.dp),
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
            Text("${bio.length}/300", color = GcTextMuted, fontSize = 11.sp, modifier = Modifier.align(Alignment.End))

            Spacer(Modifier.height(12.dp))
            Button(
                onClick = {
                    saving = true
                    scope.launch {
                        val r = withContext(Dispatchers.IO) { ApiClient.updateBio(bio) }
                        saving = false
                        r.onSuccess { saved = true }.onFailure { error = it.message }
                    }
                },
                enabled = !saving,
                modifier = Modifier.fillMaxWidth().height(48.dp),
                shape = RoundedCornerShape(12.dp),
                colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
            ) {
                Text(if (saving) "Saving…" else if (saved) "Saved ✓" else "Save bio")
            }

            Spacer(Modifier.height(24.dp))
            TextButton(onClick = {
                ApiClient.logout()
                onLogout()
            }) {
                Text("Log out", color = GcDanger)
            }
        }
    }
}

@Composable
private fun Stat(label: String, value: String) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier
            .background(GcSurface, RoundedCornerShape(12.dp))
            .border(1.dp, GcBorder, RoundedCornerShape(12.dp))
            .padding(horizontal = 16.dp, vertical = 12.dp)
    ) {
        Text(value, color = GcGold, fontWeight = FontWeight.Bold, fontSize = 18.sp)
        Text(label, color = GcTextMuted, fontSize = 11.sp)
    }
}
