package com.geniusclan.app.ui.screens.settings

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
fun TwoFactorScreen(onBack: () -> Unit, initiallyEnabled: Boolean = false) {
    var enabled by remember { mutableStateOf(initiallyEnabled) }
    var password by remember { mutableStateOf("") }
    var code by remember { mutableStateOf("") }
    var confirm by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    var info by remember { mutableStateOf<String?>(null) }
    var loading by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    Column(
        modifier = Modifier.fillMaxSize().background(GcBg).padding(20.dp)
    ) {
        TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
        Text("Two-step verification", color = GcText, fontWeight = FontWeight.Bold, fontSize = 22.sp)
        Text(
            if (enabled) "Enter password + code to turn off."
            else "Choose a 6-digit code used on new devices.",
            color = GcTextMuted,
            fontSize = 13.sp
        )
        Spacer(Modifier.height(16.dp))
        Field(password, { password = it }, "Current password", true)
        Spacer(Modifier.height(10.dp))
        Field(code, { code = it.filter { ch -> ch.isDigit() }.take(6) }, "6-digit code")
        if (!enabled) {
            Spacer(Modifier.height(10.dp))
            Field(confirm, { confirm = it.filter { ch -> ch.isDigit() }.take(6) }, "Confirm code")
        }
        error?.let { Text(it, color = GcDanger, fontSize = 13.sp, modifier = Modifier.padding(top = 8.dp)) }
        info?.let { Text(it, color = GcGold, fontSize = 13.sp, modifier = Modifier.padding(top = 8.dp)) }
        Spacer(Modifier.height(16.dp))
        Button(
            onClick = {
                loading = true
                error = null
                info = null
                scope.launch {
                    val r = withContext(Dispatchers.IO) {
                        if (enabled) ApiClient.disable2FA(password, code)
                        else {
                            if (code != confirm) Result.failure(Exception("Codes do not match"))
                            else ApiClient.enable2FA(password, code, confirm)
                        }
                    }
                    loading = false
                    r.onSuccess {
                        enabled = !enabled
                        info = if (enabled) "2FA is on" else "2FA is off"
                        password = ""; code = ""; confirm = ""
                    }.onFailure { error = it.message }
                }
            },
            enabled = !loading,
            modifier = Modifier.fillMaxWidth().height(48.dp),
            shape = RoundedCornerShape(12.dp),
            colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
        ) {
            Text(if (loading) "Please wait…" else if (enabled) "Turn off" else "Turn on")
        }
    }
}

@Composable
private fun Field(value: String, onChange: (String) -> Unit, label: String, password: Boolean = false) {
    OutlinedTextField(
        value = value,
        onValueChange = onChange,
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
            cursorColor = GcGold,
            focusedLabelColor = GcGold,
            unfocusedLabelColor = GcTextMuted
        )
    )
}
