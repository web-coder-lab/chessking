package com.geniusclan.app.data.api

import android.content.Context

/** Persist JWT so user stays logged in across app restarts. */
object TokenStore {
    private const val PREFS = "genius_clan_auth"
    private const val KEY_ACCESS = "access_token"
    private const val KEY_REFRESH = "refresh_token"

    fun load(context: Context) {
        val p = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        ApiClient.accessToken = p.getString(KEY_ACCESS, null)
        ApiClient.refreshToken = p.getString(KEY_REFRESH, null)
    }

    fun save(context: Context, access: String?, refresh: String?) {
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit()
            .putString(KEY_ACCESS, access)
            .putString(KEY_REFRESH, refresh)
            .apply()
        ApiClient.accessToken = access
        ApiClient.refreshToken = refresh
    }

    fun clear(context: Context) {
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit().clear().apply()
        ApiClient.logout()
    }

    fun hasSession(context: Context): Boolean {
        load(context)
        return !ApiClient.accessToken.isNullOrBlank()
    }
}
