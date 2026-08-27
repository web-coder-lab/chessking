package com.geniusclan.app.ui.screens.home

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AccountCircle
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.SportsEsports
import androidx.compose.material.icons.filled.Wallet
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier.modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.geniusclan.app.ui.theme.GcBg
import com.geniusclan.app.ui.theme.GcBorder
import com.geniusclan.app.ui.theme.GcGold
import com.geniusclan.app.ui.theme.GcGoldSoft
import com.geniusclan.app.ui.theme.GcSurface
import com.geniusclan.app.ui.theme.GcText
import com.geniusclan.app.ui.theme.GcTextMuted

@Composable
fun HomeScreen(
    title: String = "Home",
    subtitle: String = "Native Jetpack Compose · no WebView",
    onPlay: () -> Unit,
    onWallet: () -> Unit,
    onProfile: () -> Unit,
    onSettings: () -> Unit = {},
    onShop: () -> Unit = {},
    onInventory: () -> Unit = {}
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .padding(horizontal = 20.dp, vertical = 16.dp)
    ) {
        Text(text = "Genius Clan", color = GcGold, fontWeight = FontWeight.Bold, fontSize = 14.sp)
        Text(text = title, color = GcText, fontWeight = FontWeight.Bold, fontSize = 28.sp)
        Text(text = subtitle, color = GcTextMuted, fontSize = 13.sp)
        Spacer(Modifier.height(24.dp))

        FeatureCard("⚡ Quick Match", "Ranked & casual — Phase 4", onPlay)
        Spacer(Modifier.height(12.dp))
        FeatureCard("🪙 Wallet", "Coins & checkout — Phase 3", onWallet)
        Spacer(modifier = Modifier.height(12.dp))
        FeatureCard("👤 Profile", "Edit bio & stats", onProfile)
        Spacer(Modifier.height(12.dp))
        FeatureCard("🛒 Shop", "Buy avatars & banners", onShop)
        Spacer(Modifier.height(12.dp))
        FeatureCard("🎒 Inventory", "Equip owned items", onInventory)
        Spacer(Modifier.height(12.dp))
        FeatureCard("⚙ Settings", "2FA, sessions, support, legal", onSettings)

        Spacer(Modifier = Modifier.weight(1f))
        BottomBar(onPlay = onPlay, onWallet = onWallet, onProfile = onProfile)
    }
}

@Composable
private fun FeatureCard(title: String, subtitle: String, onClick: () -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(16.dp))
            .background(GcSurface)
            .border(1.dp, GcBorder, RoundedCornerShape(16.dp))
            .clickable(onClick = onClick)
            .padding(18.dp)
    ) {
        Text(text = title, color = GcText, fontWeight = FontWeight.SemiBold, fontSize = 17.sp)
        Text(text = subtitle, color = GcTextMuted, fontSize = 13.sp, modifier = Modifier.padding(top = 4.dp))
    }
}

@Composable
private fun BottomBar(onPlay: () -> Unit, onWallet: () -> Unit, onProfile: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(20.dp))
            .background(GcSurface)
            .border(1.dp, GcGoldSoft, RoundedCornerShape(20.dp))
            .padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceEvenly
    ) {
        NavItem(Icons.Default.Home, "Home", onClick = {})
        NavItem(Icons.Default.SportsEsports, "Play", onClick = onPlay)
        NavItem(Icons.Default.Wallet, "Wallet", onClick = onWallet)
        NavItem(Icons.Default.AccountCircle, "Profile", onClick = onProfile)
    }
}

@Composable
private fun NavItem(icon: ImageVector, label: String, onClick: () -> Unit) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier
            .clickable(onClick = onClick)
            .padding(8.dp)
    ) {
        Icon(icon, contentDescription = label, tint = GcGold, modifier = Modifier.size(24.dp))
        Text(text = label, color = GcTextMuted, fontSize = 11.sp)
    }
}
