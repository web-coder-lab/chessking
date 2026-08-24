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

    fun login(identifier: String, password: String): Result<Unit> {
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
                    return Result.failure(Exception("2FA required — enter code on web for now"))
                }
                accessToken = obj.optString("access_token").ifBlank { null }
                refreshToken = obj.optString("refresh_token").ifBlank { null }
                if (accessToken == null) return Result.failure(Exception("No access token"))
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

    fun logout() {
        accessToken = null
        refreshToken = null
    }
}
