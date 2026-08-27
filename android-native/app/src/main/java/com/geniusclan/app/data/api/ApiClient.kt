package com.geniusclan.app.data.api

import com.geniusclan.app.BuildConfig
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject
import java.util.concurrent.TimeUnit

data class ProfileDto(
    val username: String,
    val email: String?,
    val bio: String?,
    val rating: Int,
    val coinBalance: Long,
    val wins: Int = 0,
    val losses: Int = 0
)

data class WalletDto(
    val coinBalance: Long,
    val raw: String = ""
)

object ApiClient {
    private val jsonMedia = "application/json; charset=utf-8".toMediaType()
    private val client = OkHttpClient.Builder()
        .connectTimeout(25, TimeUnit.SECONDS)
        .readTimeout(25, TimeUnit.SECONDS)
        .build()

    @Volatile var accessToken: String? = null
    @Volatile var refreshToken: String? = null

    private fun api(path: String) = "${BuildConfig.API_BASE_URL}/api/v1$path"

    private fun errorMessage(text: String, code: Int): String {
        return runCatching {
            val o = JSONObject(text)
            o.optJSONObject("error")?.optString("message")?.ifBlank { null }
                ?: o.optString("message").ifBlank { null }
                ?: o.optJSONObject("error")?.optString("code")?.ifBlank { null }
        }.getOrNull() ?: "Request failed ($code)"
    }

    private fun authed(builder: Request.Builder): Request.Builder {
        accessToken?.let { builder.header("Authorization", "Bearer $it") }
        return builder
    }


    sealed class LoginResult {
        data object Success : LoginResult()
        data class Needs2FA(val pendingId: String) : LoginResult()
    }

    fun login(identifier: String, password: String): Result<LoginResult> {
        return try {
            val body = JSONObject()
                .put("identifier", identifier)
                .put("password", password)
                .toString()
                .toRequestBody(jsonMedia)
            val req = Request.Builder()
                .url(api("/auth/login"))
                .post(body)
                .header("Content-Type", "application/json")
                .build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val obj = JSONObject(text)
                if (obj.optBoolean("requires_2fa")) {
                    val pending = obj.optString("pending_id")
                    if (pending.isBlank()) return Result.failure(Exception("2FA pending id missing"))
                    return Result.success(LoginResult.Needs2FA(pending))
                }
                accessToken = obj.optString("access_token").ifBlank { null }
                refreshToken = obj.optString("refresh_token").ifBlank { null }
                if (accessToken == null) return Result.failure(Exception("No access token"))
                Result.success(LoginResult.Success)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun login2FA(pendingId: String, code: String): Result<Unit> {
        return try {
            val body = JSONObject()
                .put("pending_id", pendingId)
                .put("code", code)
                .toString()
                .toRequestBody(jsonMedia)
            val req = Request.Builder()
                .url(api("/auth/login/2fa"))
                .post(body)
                .header("Content-Type", "application/json")
                .build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val obj = JSONObject(text)
                accessToken = obj.optString("access_token").ifBlank { null }
                refreshToken = obj.optString("refresh_token").ifBlank { null }
                if (accessToken == null) return Result.failure(Exception("No access token"))
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun forgotPassword(email: String): Result<String> {
        return try {
            val body = JSONObject().put("email", email).toString().toRequestBody(jsonMedia)
            val req = Request.Builder()
                .url(api("/auth/forgot-password"))
                .post(body)
                .header("Content-Type", "application/json")
                .build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                Result.success("If that email is registered, a reset link was sent.")
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun resetPassword(token: String, newPassword: String): Result<Unit> {
        return try {
            val body = JSONObject()
                .put("token", token)
                .put("new_password", newPassword)
                .toString()
                .toRequestBody(jsonMedia)
            val req = Request.Builder()
                .url(api("/auth/reset-password"))
                .post(body)
                .header("Content-Type", "application/json")
                .build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun completeSignup(token: String, username: String, password: String): Result<Unit> {
        return try {
            val body = JSONObject()
                .put("token", token)
                .put("username", username)
                .put("password", password)
                .toString()
                .toRequestBody(jsonMedia)
            val req = Request.Builder()
                .url(api("/auth/complete-signup"))
                .post(body)
                .header("Content-Type", "application/json")
                .build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val obj = JSONObject(text)
                accessToken = obj.optString("access_token").ifBlank { null }
                refreshToken = obj.optString("refresh_token").ifBlank { null }
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }


    /** Email-first signup intent (sends mail when SMTP configured). */
    fun registerIntent(email: String): Result<String> {
        return try {
            val body = JSONObject().put("email", email).toString().toRequestBody(jsonMedia)
            val req = Request.Builder()
                .url(api("/auth/register-intent"))
                .post(body)
                .header("Content-Type", "application/json")
                .build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val msg = runCatching {
                    JSONObject(text).optString("message").ifBlank {
                        JSONObject(text).optString("status", "Check your email to complete signup")
                    }
                }.getOrDefault("Check your email to complete signup")
                Result.success(msg)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    /** Direct register if API supports it. */
    fun register(username: String, email: String, password: String): Result<Unit> {
        return try {
            val body = JSONObject()
                .put("username", username)
                .put("email", email)
                .put("password", password)
                .toString()
                .toRequestBody(jsonMedia)
            val req = Request.Builder()
                .url(api("/auth/register"))
                .post(body)
                .header("Content-Type", "application/json")
                .build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                // Some flows need email verify; try login after
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun getMyProfile(): Result<ProfileDto> {
        return try {
            val req = authed(
                Request.Builder().url(api("/profile/me")).get()
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val o = JSONObject(text)
                Result.success(
                    ProfileDto(
                        username = o.optString("username", "Player"),
                        email = o.optString("email").ifBlank { null },
                        bio = o.optString("bio").ifBlank { null },
                        rating = o.optInt("rating", 1200),
                        coinBalance = o.optLong("coin_balance", o.optLong("coins", 0)),
                        wins = o.optInt("wins", 0),
                        losses = o.optInt("losses", 0)
                    )
                )
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun updateBio(bio: String): Result<Unit> {
        return try {
            val body = JSONObject().put("bio", bio).toString().toRequestBody(jsonMedia)
            val req = authed(
                Request.Builder()
                    .url(api("/profile/me"))
                    .patch(body)
                    .header("Content-Type", "application/json")
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun getWalletBalance(): Result<WalletDto> {
        return try {
            val req = authed(
                Request.Builder().url(api("/wallet/balance")).get()
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) {
                    // Fallback: coins from profile
                    val p = getMyProfile()
                    return p.map { WalletDto(it.coinBalance) }
                }
                val o = JSONObject(text)
                val bal = when {
                    o.has("coin_balance") -> o.optLong("coin_balance")
                    o.has("balance") -> o.optLong("balance")
                    o.has("coins") -> o.optLong("coins")
                    else -> 0L
                }
                Result.success(WalletDto(bal, text))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }


    fun enable2FA(currentPassword: String, code: String, confirmCode: String): Result<Unit> {
        return try {
            val body = JSONObject()
                .put("current_password", currentPassword)
                .put("new_code", code)
                .put("confirm_code", confirmCode)
                .toString()
                .toRequestBody(jsonMedia)
            val req = authed(
                Request.Builder().url(api("/auth/2fa/enable")).post(body)
                    .header("Content-Type", "application/json")
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun disable2FA(currentPassword: String, code: String): Result<Unit> {
        return try {
            val body = JSONObject()
                .put("current_password", currentPassword)
                .put("current_code", code)
                .toString()
                .toRequestBody(jsonMedia)
            val req = authed(
                Request.Builder().url(api("/auth/2fa/disable")).post(body)
                    .header("Content-Type", "application/json")
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    data class SessionDto(val id: String, val label: String, val current: Boolean)

    fun getSessions(): Result<List<SessionDto>> {
        return try {
            val req = authed(Request.Builder().url(api("/auth/sessions")).get()).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val list = mutableListOf<SessionDto>()
                val root = org.json.JSONTokener(text).nextValue()
                val arr = when (root) {
                    is org.json.JSONArray -> root
                    is JSONObject -> root.optJSONArray("sessions") ?: root.optJSONArray("data")
                    else -> null
                }
                if (arr != null) {
                    for (i in 0 until arr.length()) {
                        val o = arr.optJSONObject(i) ?: continue
                        list.add(
                            SessionDto(
                                id = o.optString("id", o.optString("session_id")),
                                label = o.optString("device", o.optString("user_agent", "Session")).ifBlank { "Session" },
                                current = o.optBoolean("current", o.optBoolean("is_current"))
                            )
                        )
                    }
                }
                Result.success(list)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun revokeSession(sessionId: String): Result<Unit> {
        return try {
            val req = authed(
                Request.Builder().url(api("/auth/sessions/$sessionId")).delete()
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun changePassword(currentPassword: String, newPassword: String): Result<Unit> {
        return try {
            val body = JSONObject()
                .put("current_password", currentPassword)
                .put("new_password", newPassword)
                .toString()
                .toRequestBody(jsonMedia)
            val req = authed(
                Request.Builder().url(api("/profile/me/password")).post(body)
                    .header("Content-Type", "application/json")
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun changeEmail(currentPassword: String, newEmail: String): Result<Unit> {
        return try {
            val body = JSONObject()
                .put("current_password", currentPassword)
                .put("new_email", newEmail)
                .toString()
                .toRequestBody(jsonMedia)
            val req = authed(
                Request.Builder().url(api("/profile/me/email")).post(body)
                    .header("Content-Type", "application/json")
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun fetchLegal(path: String): Result<String> {
        return try {
            val req = Request.Builder().url(api(path)).get().build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val content = runCatching {
                    val o = JSONObject(text)
                    o.optString("content").ifBlank {
                        o.optString("body").ifBlank { o.optString("text", text) }
                    }
                }.getOrDefault(text)
                Result.success(content)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun supportInfo(): Result<String> {
        return try {
            val req = Request.Builder().url(api("/support/info")).get().build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val msg = runCatching {
                    val o = JSONObject(text)
                    o.optString("email").ifBlank {
                        o.optString("support_email").ifBlank {
                            o.optString("message", text)
                        }
                    }
                }.getOrDefault(text)
                Result.success(msg)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun submitBugReport(title: String, description: String): Result<Unit> {
        return try {
            val body = JSONObject()
                .put("title", title)
                .put("description", description)
                .put("screenshot_url", JSONObject.NULL)
                .toString()
                .toRequestBody(jsonMedia)
            val req = authed(
                Request.Builder().url(api("/reports/bug")).post(body)
                    .header("Content-Type", "application/json")
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }


    data class ShopItemDto(
        val id: String,
        val name: String,
        val category: String,
        val priceCoins: Long,
        val description: String = ""
    )

    data class InventoryItemDto(
        val inventoryId: String,
        val shopItemId: String,
        val name: String,
        val category: String,
        val isEquipped: Boolean
    )

    fun listShopItems(category: String? = null): Result<List<ShopItemDto>> {
        return try {
            val path = if (category.isNullOrBlank()) "/shop/items" else "/shop/items?category=$category"
            val req = authed(Request.Builder().url(api(path)).get()).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val list = mutableListOf<ShopItemDto>()
                val root = org.json.JSONTokener(text).nextValue()
                val arr = when (root) {
                    is org.json.JSONArray -> root
                    is JSONObject -> root.optJSONArray("items") ?: root.optJSONArray("data")
                    else -> null
                }
                if (arr != null) {
                    for (i in 0 until arr.length()) {
                        val o = arr.optJSONObject(i) ?: continue
                        list.add(
                            ShopItemDto(
                                id = o.optString("id", o.optString("shop_item_id")),
                                name = o.optString("name", "Item"),
                                category = o.optString("category", "misc"),
                                priceCoins = o.optLong("price_coins", o.optLong("price", 0)),
                                description = o.optString("description", "")
                            )
                        )
                    }
                }
                Result.success(list)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun purchaseItem(shopItemId: String): Result<Unit> {
        return try {
            val body = JSONObject().put("shop_item_id", shopItemId).toString().toRequestBody(jsonMedia)
            val req = authed(
                Request.Builder().url(api("/shop/purchase")).post(body)
                    .header("Content-Type", "application/json")
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun listInventory(): Result<List<InventoryItemDto>> {
        return try {
            val req = authed(Request.Builder().url(api("/inventory")).get()).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val list = mutableListOf<InventoryItemDto>()
                val root = org.json.JSONTokener(text).nextValue()
                val arr = when (root) {
                    is org.json.JSONArray -> root
                    is JSONObject -> root.optJSONArray("items") ?: root.optJSONArray("inventory") ?: root.optJSONArray("data")
                    else -> null
                }
                if (arr != null) {
                    for (i in 0 until arr.length()) {
                        val o = arr.optJSONObject(i) ?: continue
                        list.add(
                            InventoryItemDto(
                                inventoryId = o.optString("inventory_id", o.optString("id")),
                                shopItemId = o.optString("shop_item_id"),
                                name = o.optString("name", "Item"),
                                category = o.optString("category", "misc"),
                                isEquipped = o.optBoolean("is_equipped") || o.optInt("is_equipped") == 1
                            )
                        )
                    }
                }
                Result.success(list)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun equipItem(inventoryId: String): Result<Unit> {
        return try {
            val req = authed(
                Request.Builder().url(api("/inventory/$inventoryId/equip")).post("{}".toRequestBody(jsonMedia))
                    .header("Content-Type", "application/json")
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun unequipItem(inventoryId: String): Result<Unit> {
        return try {
            val req = authed(
                Request.Builder().url(api("/inventory/$inventoryId/unequip")).post("{}".toRequestBody(jsonMedia))
                    .header("Content-Type", "application/json")
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }


    data class PackageDto(val id: String, val amountPkr: Long, val coins: Long, val bonusLabel: String)

    data class DepositStartDto(val transactionId: String, val redirectUrl: String)

    data class HistoryRowDto(val id: String, val label: String, val amount: String, val createdAt: String)

    fun listPackages(): Result<List<PackageDto>> {
        return try {
            val req = authed(Request.Builder().url(api("/wallet/packages")).get()).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val list = mutableListOf<PackageDto>()
                val root = org.json.JSONTokener(text).nextValue()
                val arr = when (root) {
                    is org.json.JSONArray -> root
                    is JSONObject -> root.optJSONArray("packages") ?: root.optJSONArray("data")
                    else -> null
                }
                if (arr != null) {
                    for (i in 0 until arr.length()) {
                        val o = arr.optJSONObject(i) ?: continue
                        list.add(
                            PackageDto(
                                id = o.optString("id"),
                                amountPkr = o.optLong("amount_pkr"),
                                coins = o.optLong("coins"),
                                bonusLabel = o.optString("bonus_label", "")
                            )
                        )
                    }
                }
                Result.success(list)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun initiateDeposit(amountPkr: Long, gateway: String, payerPhone: String?): Result<DepositStartDto> {
        return try {
            val body = JSONObject()
                .put("amount_pkr", amountPkr)
                .put("gateway", gateway)
                .put("idempotency_key", java.util.UUID.randomUUID().toString())
            if (!payerPhone.isNullOrBlank()) body.put("payer_phone", payerPhone)
            else body.put("payer_phone", JSONObject.NULL)
            val req = authed(
                Request.Builder().url(api("/wallet/deposit/initiate")).post(body.toString().toRequestBody(jsonMedia))
                    .header("Content-Type", "application/json")
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val o = JSONObject(text)
                Result.success(
                    DepositStartDto(
                        transactionId = o.optString("payment_transaction_id"),
                        redirectUrl = o.optString("redirect_url")
                    )
                )
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun depositStatus(transactionId: String): Result<Pair<String, Long?>> {
        return try {
            val req = authed(
                Request.Builder().url(api("/wallet/deposit/$transactionId/status")).get()
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val o = JSONObject(text)
                val coins = if (o.isNull("coins_credited")) null else o.optLong("coins_credited")
                Result.success(o.optString("status") to coins)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun walletHistory(): Result<List<HistoryRowDto>> {
        return try {
            val req = authed(Request.Builder().url(api("/wallet/history")).get()).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val list = mutableListOf<HistoryRowDto>()
                val root = org.json.JSONTokener(text).nextValue()
                val arr = when (root) {
                    is org.json.JSONArray -> root
                    is JSONObject -> root.optJSONArray("items") ?: root.optJSONArray("transactions") ?: root.optJSONArray("history") ?: root.optJSONArray("data")
                    else -> null
                }
                if (arr != null) {
                    for (i in 0 until arr.length()) {
                        val o = arr.optJSONObject(i) ?: continue
                        list.add(
                            HistoryRowDto(
                                id = o.optString("id"),
                                label = o.optString("label", o.optString("log_type", "Transaction")),
                                amount = o.optString("amount", o.optString("coins", o.optString("delta", ""))),
                                createdAt = o.optString("created_at", "")
                            )
                        )
                    }
                }
                Result.success(list)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }


    data class DailyStatusDto(val claimedToday: Boolean, val streakDay: Long, val nextCoins: Long)
    data class LeaderboardEntryDto(val rank: Long, val username: String, val rating: Long)
    data class NotificationDto(val id: String, val title: String, val body: String, val read: Boolean)
    data class ReferralDto(val code: String, val shareUrl: String)

    fun dailyStatus(): Result<DailyStatusDto> {
        return try {
            val req = authed(Request.Builder().url(api("/rewards/daily-status")).get()).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val o = JSONObject(text)
                Result.success(
                    DailyStatusDto(
                        claimedToday = o.optBoolean("claimed_today"),
                        streakDay = o.optLong("current_streak_day"),
                        nextCoins = o.optLong("next_reward_coins")
                    )
                )
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun claimDaily(): Result<Long> {
        return try {
            val req = authed(
                Request.Builder().url(api("/rewards/daily-claim")).post("{}".toRequestBody(jsonMedia))
                    .header("Content-Type", "application/json")
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                Result.success(JSONObject(text).optLong("coins_awarded"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun leaderboard(scope: String = "global"): Result<Pair<List<LeaderboardEntryDto>, Long?>> {
        return try {
            val req = authed(Request.Builder().url(api("/leaderboard?scope=$scope")).get()).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val o = JSONObject(text)
                val arr = o.optJSONArray("rankings") ?: org.json.JSONArray()
                val list = mutableListOf<LeaderboardEntryDto>()
                for (i in 0 until arr.length()) {
                    val r = arr.optJSONObject(i) ?: continue
                    list.add(
                        LeaderboardEntryDto(
                            rank = r.optLong("rank"),
                            username = r.optString("username"),
                            rating = r.optLong("rating")
                        )
                    )
                }
                val my = if (o.isNull("my_rank")) null else o.optLong("my_rank")
                Result.success(list to my)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun notifications(): Result<List<NotificationDto>> {
        return try {
            val req = authed(Request.Builder().url(api("/notifications")).get()).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val root = org.json.JSONTokener(text).nextValue()
                val arr = when (root) {
                    is org.json.JSONArray -> root
                    is JSONObject -> root.optJSONArray("notifications") ?: root.optJSONArray("data")
                    else -> null
                } ?: org.json.JSONArray()
                val list = mutableListOf<NotificationDto>()
                for (i in 0 until arr.length()) {
                    val n = arr.optJSONObject(i) ?: continue
                    list.add(
                        NotificationDto(
                            id = n.optString("id"),
                            title = n.optString("title", n.optString("type", "Notice")),
                            body = n.optString("body", n.optString("message", "")),
                            read = n.optBoolean("is_read") || n.optBoolean("read")
                        )
                    )
                }
                Result.success(list)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun markNotificationRead(id: String): Result<Unit> {
        return try {
            val req = authed(
                Request.Builder().url(api("/notifications/$id/read")).post("{}".toRequestBody(jsonMedia))
                    .header("Content-Type", "application/json")
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun referralLink(): Result<ReferralDto> {
        return try {
            val req = authed(Request.Builder().url(api("/referral/link")).get()).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val o = JSONObject(text)
                Result.success(
                    ReferralDto(
                        code = o.optString("invite_link_code"),
                        shareUrl = o.optString("share_url")
                    )
                )
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }


    data class UserHitDto(val id: String, val username: String)
    data class InviteRowDto(val id: String, val label: String, val status: String)
    data class MatchHistoryDto(val id: String, val result: String, val opponent: String)

    fun searchCustomMatch(username: String): Result<List<UserHitDto>> {
        return try {
            val q = java.net.URLEncoder.encode(username, "UTF-8")
            val req = authed(Request.Builder().url(api("/custom-match/search?username=$q")).get()).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val o = JSONObject(text)
                val arr = o.optJSONArray("results") ?: org.json.JSONArray()
                val list = mutableListOf<UserHitDto>()
                for (i in 0 until arr.length()) {
                    val r = arr.optJSONObject(i) ?: continue
                    list.add(UserHitDto(r.optString("id"), r.optString("username")))
                }
                Result.success(list)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun sendCustomInvite(receiverUsername: String): Result<String> {
        return try {
            val body = JSONObject().put("receiver_username", receiverUsername).toString().toRequestBody(jsonMedia)
            val req = authed(
                Request.Builder().url(api("/custom-match/invite")).post(body)
                    .header("Content-Type", "application/json")
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val o = JSONObject(text)
                Result.success(o.optString("id", o.optString("invite_id", "ok")))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun respondInvite(inviteId: String, decision: String): Result<String?> {
        return try {
            val body = JSONObject().put("decision", decision).toString().toRequestBody(jsonMedia)
            val req = authed(
                Request.Builder().url(api("/custom-match/invite/$inviteId/respond")).post(body)
                    .header("Content-Type", "application/json")
            ).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val o = JSONObject(text)
                val matchId = o.optString("match_id").ifBlank { null }
                Result.success(matchId)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun inviteHistory(): Result<List<InviteRowDto>> {
        return try {
            val req = authed(Request.Builder().url(api("/custom-match/history")).get()).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val root = org.json.JSONTokener(text).nextValue()
                val arr = when (root) {
                    is org.json.JSONArray -> root
                    is JSONObject -> root.optJSONArray("invites") ?: root.optJSONArray("data")
                    else -> null
                } ?: org.json.JSONArray()
                val list = mutableListOf<InviteRowDto>()
                for (i in 0 until arr.length()) {
                    val o = arr.optJSONObject(i) ?: continue
                    list.add(
                        InviteRowDto(
                            id = o.optString("id"),
                            label = o.optString("opponent_username", o.optString("receiver_username", o.optString("sender_username", "Invite"))),
                            status = o.optString("status", "")
                        )
                    )
                }
                Result.success(list)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun matchHistory(username: String): Result<List<MatchHistoryDto>> {
        return try {
            val q = java.net.URLEncoder.encode(username, "UTF-8")
            val req = authed(Request.Builder().url(api("/profile/$q/match-history?limit=30")).get()).build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) return Result.failure(Exception(errorMessage(text, res.code)))
                val root = org.json.JSONTokener(text).nextValue()
                val arr = when (root) {
                    is org.json.JSONArray -> root
                    is JSONObject -> root.optJSONArray("matches") ?: root.optJSONArray("history") ?: root.optJSONArray("data")
                    else -> null
                } ?: org.json.JSONArray()
                val list = mutableListOf<MatchHistoryDto>()
                for (i in 0 until arr.length()) {
                    val o = arr.optJSONObject(i) ?: continue
                    list.add(
                        MatchHistoryDto(
                            id = o.optString("id", o.optString("match_id")),
                            result = o.optString("result", o.optString("status", "")),
                            opponent = o.optString("opponent", o.optString("opponent_username", "—"))
                        )
                    )
                }
                Result.success(list)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun logout() {
        accessToken = null
        refreshToken = null
    }
}
