package com.geniusclan.app.ui.screens.shop

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
import androidx.compose.material3.CircularProgressIndicator
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
import com.geniusclan.app.data.api.ApiClient.InventoryItemDto
import com.geniusclan.app.ui.theme.GcBg
import com.geniusclan.app.ui.theme.GcBorder
import com.geniusclan.app.ui.theme.GcDanger
import com.geniusclan.app.ui.theme.GcGold
import com.geniusclan.app.ui.theme.GcSuccess
import com.geniusclan.app.ui.theme.GcSurface
import com.geniusclan.app.ui.theme.GcText
import com.geniusclan.app.ui.theme.GcTextMuted
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@Composable
fun InventoryScreen(onBack: () -> Unit, onOpenShop: () -> Unit) {
    var items by remember { mutableStateOf(listOf<InventoryItemDto>()) }
    var loading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }
    var message by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()

    fun load() {
        loading = true
        error = null
        scope.launch {
            val r = withContext(Dispatchers.IO) { ApiClient.listInventory() }
            loading = false
            r.onSuccess { items = it }.onFailure { error = it.message }
        }
    }

    LaunchedEffect(Unit) { load() }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .verticalScroll(rememberScrollState())
            .padding(20.dp)
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
            Text("Inventory", color = GcText, fontWeight = FontWeight.Bold, fontSize = 24.sp, modifier = Modifier.weight(1f))
            TextButton(onClick = onOpenShop) { Text("Shop", color = GcGold) }
        }
        Text("Equip avatars, banners, boards", color = GcTextMuted, fontSize = 13.sp)
        Spacer(Modifier.height(12.dp))

        if (loading) CircularProgressIndicator(color = GcGold, modifier = Modifier.align(Alignment.CenterHorizontally))
        error?.let { Text(it, color = GcDanger) }
        message?.let { Text(it, color = GcGold, fontSize = 13.sp) }

        items.forEach { item ->
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 10.dp)
                    .background(GcSurface, RoundedCornerShape(14.dp))
                    .border(1.dp, if (item.isEquipped) GcGold else GcBorder, RoundedCornerShape(14.dp))
                    .padding(14.dp)
            ) {
                Text(item.name, color = GcText, fontWeight = FontWeight.SemiBold, fontSize = 16.sp)
                Text(item.category, color = GcTextMuted, fontSize = 12.sp)
                if (item.isEquipped) {
                    Text("Equipped", color = GcSuccess, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                }
                Row(modifier = Modifier.padding(top = 8.dp)) {
                    if (!item.isEquipped) {
                        Button(
                            onClick = {
                                scope.launch {
                                    val r = withContext(Dispatchers.IO) { ApiClient.equipItem(item.inventoryId) }
                                    r.onSuccess { message = "Equipped ${item.name}"; load() }
                                        .onFailure { error = it.message }
                                }
                            },
                            shape = RoundedCornerShape(10.dp),
                            colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
                        ) { Text("Equip") }
                    } else {
                        Button(
                            onClick = {
                                scope.launch {
                                    val r = withContext(Dispatchers.IO) { ApiClient.unequipItem(item.inventoryId) }
                                    r.onSuccess { message = "Unequipped"; load() }
                                        .onFailure { error = it.message }
                                }
                            },
                            shape = RoundedCornerShape(10.dp),
                            colors = ButtonDefaults.buttonColors(containerColor = GcSurface, contentColor = GcGold)
                        ) { Text("Unequip") }
                    }
                }
            }
        }

        if (!loading && items.isEmpty() && error == null) {
            Text("Inventory empty — buy items in the Shop.", color = GcTextMuted)
        }
    }
}
