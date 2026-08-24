package com.geniusclan.app.ui.screens.auth

import androidx.compose.foundation.background
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
    var modeRegister by remember { mutableStateOf(false) }
    var identifier by remember { mutableStateOf("") }
    var email by remember { mutableStateOf("") }
    var username by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    var info by remember { mutableStateOf<String?>(null) }
    var loading by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .padding(horizontal = 24.dp, vertical = 28.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(text = "♚", fontSize = 40.sp, color = GcGold)
        Text(
            text = if (modeRegister) "Create account" else "Welcome back",
            color = GcText,
            fontWeight = FontWeight.Bold,
            fontSize = 24.sp,
            modifier = Modifier.padding(top = 8.dp)
        )
        Text(
            text = if (modeRegister) "Join Genius Clan" else "Sign in to continue",
            color = GcTextMuted,
            fontSize = 14.sp,
            modifier = Modifier.padding(bottom = 20.dp)
        )

        if (modeRegister) {
            GcField(email, { email = it; error = null }, "Email")
            Spacer(Modifier.height(10.dp))
            GcField(username, { username = it; error = null }, "Username")
            Spacer(Modifier.height(10.dp))
            GcField(password, { password = it; error = null }, "Password", password = true)
        } else {
            GcField(identifier, { identifier = it; error = null }, "Username or email")
            Spacer(Modifier.height(10.dp))
            GcField(password, { password = it; error = null }, "Password", password = true)
        }

        if (error != null) {
            Text(text = error!!, color = GcDanger, fontSize = 13.sp, modifier = Modifier.fillMaxWidth().padding(top = 10.dp))
        }
        if (info != null) {
            Text(text = info!!, color = GcGold, fontSize = 13.sp, modifier = Modifier.fillMaxWidth().padding(top = 10.dp))
        }

        Spacer(Modifier.height(20.dp))
        Button(
            onClick = {
                loading = true
                error = null
                info = null
                scope.launch {
                    val result = withContext(Dispatchers.IO) {
                        if (modeRegister) {
                            if (email.isBlank() || username.isBlank() || password.length < 6) {
                                return@withContext Result.failure(Exception("Fill email, username, password (6+ chars)"))
                            }
                            ApiClient.register(username.trim(), email.trim(), password).mapCatching {
                                ApiClient.login(username.trim(), password).getOrThrow()
                            }
                        } else {
                            if (identifier.isBlank() || password.isBlank()) {
                                return@withContext Result.failure(Exception("Enter username and password"))
                            }
                            ApiClient.login(identifier.trim(), password)
                        }
                    }
                    loading = false
                    result.onSuccess { onLoggedIn() }
                        .onFailure { error = it.message ?: "Failed" }
                }
            },
            enabled = !loading,
            modifier = Modifier.fillMaxWidth().height(52.dp),
            shape = RoundedCornerShape(14.dp),
            colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
        ) {
            Text(
                text = when {
                    loading -> "Please wait…"
                    modeRegister -> "Create account"
                    else -> "Sign in"
                },
                fontWeight = FontWeight.SemiBold,
                fontSize = 16.sp
            )
        }

        TextButton(onClick = {
            modeRegister = !modeRegister
            error = null
            info = null
        }) {
            Text(
                text = if (modeRegister) "Already have an account? Sign in" else "Create account",
                color = GcTextMuted
            )
        }

        if (modeRegister) {
            TextButton(onClick = {
                if (email.isBlank()) {
                    error = "Enter email first"
                    return@TextButton
                }
                loading = true
                scope.launch {
                    val r = withContext(Dispatchers.IO) { ApiClient.registerIntent(email.trim()) }
                    loading = false
                    r.onSuccess { info = it }.onFailure { error = it.message }
                }
            }) {
                Text(text = "Or email invite signup", color = GcGold, fontSize = 13.sp)
            }
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
