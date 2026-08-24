package com.geniusclan.app.data.api

import com.geniusclan.app.BuildConfig
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject
import java.util.concurrent.TimeUnit

object ApiClient {
    private val json = "application/json; charset=utf-8".toMediaType()
    private val client = OkHttpClient.Builder()
        .connectTimeout(20, TimeUnit.SECONDS)
        .readTimeout(20, TimeUnit.SECONDS)
        .build()

    @Volatile
    var accessToken: String? = null

    @Volatile
    var refreshToken: String? = null

    fun login(identifier: String, password: String): Result<Unit> {
        return try {
            val body = JSONObject()
                .put("identifier", identifier)
                .put("password", password)
                .toString()
                .toRequestBody(json)
            val req = Request.Builder()
                .url("${BuildConfig.API_BASE_URL}/api/v1/auth/login")
                .post(body)
                .header("Content-Type", "application/json")
                .build()
            client.newCall(req).execute().use { res ->
                val text = res.body?.string().orEmpty()
                if (!res.isSuccessful) {
                    val msg = runCatching {
                        JSONObject(text).optJSONObject("error")?.optString("message")
                            ?: JSONObject(text).optString("message")
                    }.getOrNull()?.ifBlank { null }
                    return Result.failure(Exception(msg ?: "Login failed (${res.code})"))
                }
                val obj = JSONObject(text)
                accessToken = obj.optString("access_token").ifBlank { null }
                refreshToken = obj.optString("refresh_token").ifBlank { null }
                if (obj.optBoolean("requires_2fa")) {
                    return Result.failure(Exception("2FA required — Phase 3 UI"))
                }
                if (accessToken == null) {
                    return Result.failure(Exception("No access token in response"))
                }
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}
