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
fun ChangePasswordScreen(onBack: () -> Unit) {
    var current by remember { mutableStateOf("") }
    var next by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    var ok by remember { mutableStateOf(false) }
    var loading by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    Column(Modifier = Modifier.fillMaxSize().background(GcBg).padding(20.dp)) {
        TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
        Text("Change password", color = GcText, fontWeight = FontWeight.Bold, fontSize = 22.sp)
        Spacer(Modifier.height(16.dp))
        PwField(current, { current = it }, "Current password")
        Spacer(Modifier.height(10.dp))
        PwField(next, { next = it }, "New password")
        error?.let { Text(it, color = GcDanger, modifier = Modifier.padding(top = 8.dp)) }
        if (ok) Text("Password updated", color = GcGold, modifier = Modifier.padding(top = 8.dp))
        Spacer(Modifier.height(16.dp))
        Button(
            onClick = {
                loading = true; error = null; ok = false
                scope.launch {
                    val r = withContext(Dispatchers.IO) { ApiClient.changePassword(current, next) }
                    loading = false
                    r.onSuccess { ok = true; current = ""; next = "" }
                        .onFailure { error = it.message }
                }
            },
            enabled = !loading,
            modifier = Modifier.fillMaxWidth().height(48.dp),
            shape = RoundedCornerShape(12.dp),
            colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
        ) { Text(if (loading) "Saving…" else "Save") }
    }
}

@Composable
private fun PwField(v: String, on: (String) -> Unit, label: String) {
    OutlinedTextField(
        value = v,
        onValueChange = on,
        label = { Text(label) },
        singleLine = true,
        visualTransformation = PasswordVisualTransformation(),
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
