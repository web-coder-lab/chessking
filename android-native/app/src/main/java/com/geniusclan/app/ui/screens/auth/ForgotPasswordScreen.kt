package com.geniusclan.app.ui.screens.auth

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
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
fun ForgotPasswordScreen(onBack: () -> Unit, onHaveToken: () -> Unit) {
    var email by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    var info by remember { mutableStateOf<String?>(null) }
    var loading by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    Column(modifier = Modifier.fillMaxSize().background(GcBg).padding(24.dp)) {
        TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
        Text("Forgot password", color = GcText, fontWeight = FontWeight.Bold, fontSize = 24.sp)
        Text("We'll email a reset link if the account exists.", color = GcTextMuted, fontSize = 13.sp)
        Spacer(Modifier.height(16.dp))
        OutlinedTextField(
            value = email,
            onValueChange = { email = it },
            label = { Text("Email") },
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
        error?.let { Text(it, color = GcDanger, modifier = Modifier.padding(top = 8.dp)) }
        info?.let { Text(it, color = GcGold, modifier = Modifier.padding(top = 8.dp)) }
        Spacer(Modifier.height(16.dp))
        Button(
            onClick = {
                if (email.isBlank()) {
                    error = "Enter email"
                    return@Button
                }
                loading = true
                error = null
                scope.launch {
                    val r = withContext(Dispatchers.IO) { ApiClient.forgotPassword(email.trim()) }
                    loading = false
                    r.onSuccess { info = it }.onFailure { error = it.message }
                }
            },
            enabled = !loading,
            modifier = Modifier.fillMaxWidth().height(50.dp),
            shape = RoundedCornerShape(12.dp),
            colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
        ) { Text(if (loading) "Sending…" else "Send reset email") }
        TextButton(onClick = onHaveToken) {
            Text("I already have a reset token", color = GcTextMuted)
        }
    }
}
