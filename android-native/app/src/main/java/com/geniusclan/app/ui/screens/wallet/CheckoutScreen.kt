package com.geniusclan.app.ui.screens.wallet

import android.content.Intent
import android.net.Uri
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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.geniusclan.app.data.api.ApiClient
import com.geniusclan.app.data.api.ApiClient.PackageDto
import com.geniusclan.app.ui.theme.GcBg
import com.geniusclan.app.ui.theme.GcBorder
import com.geniusclan.app.ui.theme.GcDanger
import com.geniusclan.app.ui.theme.GcGold
import com.geniusclan.app.ui.theme.GcSurface
import com.geniusclan.app.ui.theme.GcText
import com.geniusclan.app.ui.theme.GcTextMuted
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

private val GATEWAYS = listOf(
    "jazzcash" to "JazzCash",
    "easypaisa" to "EasyPaisa",
    "googlepay" to "Google Pay"
)

@Composable
fun CheckoutScreen(onBack: () -> Unit) {
    val context = LocalContext.current
    var packages by remember { mutableStateOf(listOf<PackageDto>()) }
    var selected by remember { mutableStateOf<PackageDto?>(null) }
    var customAmount by remember { mutableStateOf("") }
    var gateway by remember { mutableStateOf("jazzcash") }
    var phone by remember { mutableStateOf("") }
    var loading by remember { mutableStateOf(true) }
    var paying by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var phase by remember { mutableStateOf("form") } // form | pending | success | failed
    var coinsCredited by remember { mutableStateOf<Long?>(null) }
    var txId by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        val r = withContext(Dispatchers.IO) { ApiClient.listPackages() }
        loading = false
        r.onSuccess { packages = it }.onFailure { error = it.message }
    }

    LaunchedEffect(phase, txId) {
        if (phase != "pending" || txId == null) return@LaunchedEffect
        while (isActive && phase == "pending") {
            delay(3000)
            val r = withContext(Dispatchers.IO) { ApiClient.depositStatus(txId!!) }
            r.onSuccess { (status, coins) ->
                when (status) {
                    "success" -> {
                        coinsCredited = coins
                        phase = "success"
                    }
                    "failed" -> phase = "failed"
                }
            }
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(GcBg)
            .verticalScroll(rememberScrollState())
            .padding(20.dp)
    ) {
        TextButton(onClick = onBack) { Text("← Back", color = GcGold) }
        Text("Add coins", color = GcText, fontWeight = FontWeight.Bold, fontSize = 24.sp)

        when (phase) {
            "pending" -> {
                Spacer(Modifier.height(32.dp))
                CircularProgressIndicator(color = GcGold, modifier = Modifier.align(Alignment.CenterHorizontally))
                Text(
                    "Waiting for payment…",
                    color = GcText,
                    modifier = Modifier.padding(top = 16.dp).align(Alignment.CenterHorizontally)
                )
                Text(
                    "Complete payment in the browser, then wait here.",
                    color = GcTextMuted,
                    fontSize = 13.sp,
                    modifier = Modifier.padding(top = 8.dp).align(Alignment.CenterHorizontally)
                )
            }
            "success" -> {
                Spacer(Modifier.height(32.dp))
                Text("Payment confirmed", color = GcGold, fontWeight = FontWeight.Bold, fontSize = 20.sp)
                Text(
                    if (coinsCredited != null) "+$coinsCredited coins" else "Coins added",
                    color = GcText,
                    fontSize = 16.sp
                )
                Spacer(Modifier.height(16.dp))
                Button(
                    onClick = onBack,
                    colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
                ) { Text("Back to wallet") }
            }
            "failed" -> {
                Spacer(Modifier.height(32.dp))
                Text("Payment failed", color = GcDanger, fontWeight = FontWeight.Bold, fontSize = 20.sp)
                Button(
                    onClick = { phase = "form"; error = null },
                    colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
                ) { Text("Try again") }
            }
            else -> {
                if (loading) CircularProgressIndicator(color = GcGold)
                error?.let { Text(it, color = GcDanger, fontSize = 13.sp) }

                Text("Packages", color = GcGold, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                packages.forEach { pkg ->
                    val sel = selected?.id == pkg.id
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(bottom = 8.dp)
                            .background(GcSurface, RoundedCornerShape(12.dp))
                            .border(1.dp, if (sel) GcGold else GcBorder, RoundedCornerShape(12.dp))
                            .clickable {
                                selected = pkg
                                customAmount = pkg.amountPkr.toString()
                            }
                            .padding(14.dp)
                    ) {
                        Text("Rs ${pkg.amountPkr}", color = GcText, fontWeight = FontWeight.Bold)
                        Text("${pkg.coins} coins ${pkg.bonusLabel}", color = GcTextMuted, fontSize = 13.sp)
                    }
                }

                Spacer(Modifier.height(8.dp))
                Text("Or custom amount (PKR, min 100)", color = GcTextMuted, fontSize = 12.sp)
                OutlinedTextField(
                    value = customAmount,
                    onValueChange = { customAmount = it.filter { ch -> ch.isDigit() }; selected = null },
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

                Spacer(Modifier.height(12.dp))
                Text("Payment method", color = GcGold, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                GATEWAYS.forEach { (id, label) ->
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(bottom = 6.dp)
                            .background(GcSurface, RoundedCornerShape(10.dp))
                            .border(1.dp, if (gateway == id) GcGold else GcBorder, RoundedCornerShape(10.dp))
                            .clickable { gateway = id }
                            .padding(12.dp)
                    ) {
                        Text(label, color = GcText, modifier = Modifier.weight(1f))
                        if (gateway == id) Text("✓", color = GcGold)
                    }
                }

                if (gateway == "jazzcash" || gateway == "easypaisa") {
                    Spacer(Modifier.height(8.dp))
                    OutlinedTextField(
                        value = phone,
                        onValueChange = { phone = it },
                        label = { Text("Mobile 03XXXXXXXXX") },
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
                }

                Spacer(Modifier.height(16.dp))
                Button(
                    onClick = {
                        val amount = customAmount.toLongOrNull() ?: 0L
                        if (amount < 100) {
                            error = "Minimum Rs 100"
                            return@Button
                        }
                        if ((gateway == "jazzcash" || gateway == "easypaisa") && phone.length < 11) {
                            error = "Enter valid mobile number"
                            return@Button
                        }
                        paying = true
                        error = null
                        scope.launch {
                            val r = withContext(Dispatchers.IO) {
                                ApiClient.initiateDeposit(
                                    amount,
                                    gateway,
                                    if (gateway == "googlepay") null else phone
                                )
                            }
                            paying = false
                            r.onSuccess { start ->
                                txId = start.transactionId
                                phase = "pending"
                                if (start.redirectUrl.isNotBlank()) {
                                    try {
                                        context.startActivity(
                                            Intent(Intent.ACTION_VIEW, Uri.parse(start.redirectUrl))
                                        )
                                    } catch (_: Exception) {
                                    }
                                }
                            }.onFailure { error = it.message }
                        }
                    },
                    enabled = !paying,
                    modifier = Modifier.fillMaxWidth().height(50.dp),
                    shape = RoundedCornerShape(12.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = GcGold, contentColor = GcBg)
                ) {
                    Text(if (paying) "Starting…" else "Pay now", fontWeight = FontWeight.SemiBold)
                }
            }
        }
    }
}
