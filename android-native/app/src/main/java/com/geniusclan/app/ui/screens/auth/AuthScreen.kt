package com.geniusclan.app.ui.screens.auth

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
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
fun AuthScreen(onLoggedIn: () -> Unit) {
    var identifier by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    var loading by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .padding(horizontal = 24.dp, vertical = 32.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(text = "♚", fontSize = 40.sp, color = GcGold)
        Text(
            text = "Welcome back",
            color = GcText,
            fontWeight = FontWeight.Bold,
            fontSize = 24.sp,
            modifier = Modifier.padding(top = 8.dp)
        )
        Text(
            text = "Sign in to Genius Clan",
            color = GcTextMuted,
            fontSize = 14.sp,
            modifier = Modifier.padding(bottom = 28.dp)
        )

        GcField(
            value = identifier,
            onValueChange = { identifier = it; error = null },
            label = "Username or email"
        )
        Spacer(Modifier.height(12.dp))
        GcField(
            value = password,
            onValueChange = { password = it; error = null },
            label = "Password",
            password = true
        )

        if (error != null) {
            Text(
                text = error!!,
                color = GcDanger,
                fontSize = 13.sp,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 12.dp)
            )
        }

        Spacer(Modifier.height(24.dp))
        Button(
            onClick = {
                if (identifier.isBlank() || password.isBlank()) {
                    error = "Enter username and password"
                    return@Button
                }
                loading = true
                error = null
                scope.launch {
                    val result = withContext(Dispatchers.IO) {
                        ApiClient.login(identifier.trim(), password)
                    }
                    loading = false
                    result.onSuccess { onLoggedIn() }
                        .onFailure { error = it.message ?: "Login failed" }
                }
            },
            enabled = !loading,
            modifier = Modifier
                .fillMaxWidth()
                .height(52.dp),
            shape = RoundedCornerShape(14.dp),
            colors = ButtonDefaults.buttonColors(
                containerColor = GcGold,
                contentColor = GcBg
            )
        ) {
            Text(
                text = if (loading) "Signing in…" else "Sign in",
                fontWeight = FontWeight.SemiBold,
                fontSize = 16.sp
            )
        }

        TextButton(onClick = { /* Phase 3: register */ }) {
            Text(text = "Create account — Phase 3", color = GcTextMuted)
        }
    }
}

@Composable
private fun GcField(
    value: String,
    onValueChange: (String) -> Unit,
    label: String,
    password: Boolean = false
) {
    OutlinedTextField(
        value = value,
        onValueChange = onValueChange,
        label = { Text(label) },
        singleLine = true,
        visualTransformation = if (password) PasswordVisualTransformation() else androidx.compose.ui.text.input.VisualTransformation.None,
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(12.dp),
        colors = OutlinedTextFieldDefaults.colors(
            focusedBorderColor = GcGold,
            unfocusedBorderColor = GcBorder,
            focusedContainerColor = GcSurface,
            unfocusedContainerColor = GcSurface,
            focusedTextColor = GcText,
            unfocusedTextColor = GcText,
            focusedLabelColor = GcGold,
            unfocusedLabelColor = GcTextMuted,
            cursorColor = GcGold
        )
    )
}
