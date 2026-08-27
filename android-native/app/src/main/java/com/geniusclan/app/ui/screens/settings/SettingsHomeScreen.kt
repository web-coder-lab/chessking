package com.geniusclan.app.ui.screens.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
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
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.geniusclan.app.ui.theme.GcBg
import com.geniusclan.app.ui.theme.GcBorder
import com.geniusclan.app.ui.theme.GcGold
import com.geniusclan.app.ui.theme.GcSurface
import com.geniusclan.app.ui.theme.GcText
import com.geniusclan.app.ui.theme.GcTextMuted

@Composable
fun SettingsHomeScreen(
    onBack: () -> Unit,
    onOpen: (String) -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .verticalScroll(rememberScrollState())
            .padding(20.dp)
    ) {
        TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
        Text("Settings", color = GcText, fontWeight = FontWeight.Bold, fontSize = 28.sp)
        Text("Account & app", color = GcTextMuted, fontSize = 13.sp)
        Spacer(Modifier.height(16.dp))

        Section("Security")
        SettingsRow("Two-step verification", "PIN when signing in") { onOpen("2fa") }
        SettingsRow("Active sessions", "Devices logged in") { onOpen("sessions") }
        SettingsRow("Change password", null) { onOpen("password") }
        SettingsRow("Change email", null) { onOpen("email") }

        Spacer(Modifier.height(16.dp))
        Section("Support")
        SettingsRow("Bug report", null) { onOpen("bug") }
        SettingsRow("Contact support", null) { onOpen("support") }

        Spacer(Modifier.height(16.dp))
        Section("Legal")
        SettingsRow("Privacy policy", null) { onOpen("privacy") }
        SettingsRow("Terms of service", null) { onOpen("terms") }
        SettingsRow("About", null) { onOpen("about") }

        Spacer(Modifier.height(24.dp))
        Text(
            "Admin panel is web-only (not in this app).",
            color = GcTextMuted,
            fontSize = 11.sp
        )
    }
}

@Composable
private fun Section(title: String) {
    Text(
        text = title.uppercase(),
        color = GcGold,
        fontSize = 11.sp,
        fontWeight = FontWeight.SemiBold,
        letterSpacing = 1.sp,
        modifier = Modifier.padding(bottom = 8.dp, top = 4.dp)
    )
}

@Composable
private fun SettingsRow(title: String, subtitle: String?, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(bottom = 8.dp)
            .background(GcSurface, RoundedCornerShape(14.dp))
            .border(1.dp, GcBorder, RoundedCornerShape(14.dp))
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 14.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(title, color = GcText, fontWeight = FontWeight.SemiBold, fontSize = 15.sp)
            if (subtitle != null) {
                Text(subtitle, color = GcTextMuted, fontSize = 12.sp)
            }
        }
        Text("›", color = GcGold, fontSize = 20.sp)
    }
}
