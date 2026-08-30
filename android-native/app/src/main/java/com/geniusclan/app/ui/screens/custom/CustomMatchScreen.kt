package com.geniusclan.app.ui.screens.custom

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
fun CustomMatchScreen(
    onBack: () -> Unit,
    onMatchReady: (matchId: String, color: String) -> Unit
) {
    var query by remember { mutableStateOf("") }
    var hits by remember { mutableStateOf(listOf<ApiClient.UserHitDto>()) }
    var invites by remember { mutableStateOf(listOf<ApiClient.InviteRowDto>()) }
    var message by remember { mutableStateOf<String?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    var loading by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    fun loadInvites() {
        scope.launch {
            val r = withContext(Dispatchers.IO) { ApiClient.inviteHistory() }
            r.onSuccess { invites = it }.onFailure { /* soft */ }
        }
    }

    LaunchedEffect(Unit) { loadInvites() }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .verticalScroll(rememberScrollState())
            .padding(20.dp)
    ) {
        TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
        Text("Custom match", color = GcText, fontWeight = FontWeight.Bold, fontSize = 24.sp)
        Text("Search a player and send invite", color = GcTextMuted, fontSize = 13.sp)
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
                scope.launch {
                    val r = withContext(Dispatchers.IO) { ApiClient.searchCustomMatch(query.trim()) }
                    loading = false
                    r.onSuccess { hits = it }.onFailure { error = it.message }
                }
            },
            enabled = query.isNotBlank() && !loading,
            modifier = Modifier.fillMaxWidth(),
            shape = RoundedCornerShape(12.dp),
            colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
        ) { Text(if (loading) "Searching…" else "Search") }

        error?.let { Text(it, color = GcDanger, modifier = Modifier.padding(top = 8.dp)) }
        message?.let { Text(it, color = GcGold, modifier = Modifier.padding(top = 8.dp)) }

        hits.forEach { u ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 8.dp)
                    .background(GcSurface, RoundedCornerShape(12.dp))
                    .border(1.dp, GcBorder, RoundedCornerShape(12.dp))
                    .padding(12.dp)
            ) {
                Text(u.username, color = GcText, modifier = Modifier.weight(1f), fontWeight = FontWeight.SemiBold)
                TextButton(onClick = {
                    scope.launch {
                        val r = withContext(Dispatchers.IO) { ApiClient.sendCustomInvite(u.username) }
                        r.onSuccess { message = "Invite sent to ${u.username}"; loadInvites() }
                            .onFailure { error = it.message }
                    }
                }) { Text("Invite", color = GcGold) }
            }
        }

        Spacer(Modifier.height(20.dp))
        Text("Invite history", color = GcGold, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
        invites.forEach { inv ->
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 8.dp)
                    .background(GcSurface, RoundedCornerShape(12.dp))
                    .border(1.dp, GcBorder, RoundedCornerShape(12.dp))
                    .padding(12.dp)
            ) {
                Text(inv.label, color = GcText, fontWeight = FontWeight.SemiBold)
                Text(inv.status, color = GcTextMuted, fontSize = 12.sp)
                if (inv.status.equals("pending", true) && inv.id.isNotBlank()) {
                    Row {
                        TextButton(onClick = {
                            scope.launch {
                                val r = withContext(Dispatchers.IO) {
                                    ApiClient.respondInvite(inv.id, "accept")
                                }
                                r.onSuccess { matchId ->
                                    if (!matchId.isNullOrBlank()) onMatchReady(matchId, "white")
                                    else message = "Accepted"
                                    loadInvites()
                                }.onFailure { error = it.message }
                            }
                        }) { Text("Accept", color = GcGold) }
                        TextButton(onClick = {
                            scope.launch {
                                withContext(Dispatchers.IO) { ApiClient.respondInvite(inv.id, "decline") }
                                loadInvites()
                            }
                        }) { Text("Decline", color = GcDanger) }
                    }
                }
            }
        }
    }
}
