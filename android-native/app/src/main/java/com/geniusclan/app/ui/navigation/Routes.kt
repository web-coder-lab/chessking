package com.geniusclan.app.ui.navigation

object Routes {
    const val SPLASH = "splash"
    const val SERVER_GATE = "server_gate"
    const val AUTH = "auth"
    const val HOME = "home"
    const val PLAY = "play"
    const val WALLET = "wallet"
    const val PROFILE = "profile"
    const val BOARD = "board/{matchId}/{color}"
    const val SETTINGS = "settings"
    const val SETTINGS_2FA = "settings/2fa"
    const val SETTINGS_SESSIONS = "settings/sessions"
    const val SETTINGS_PASSWORD = "settings/password"
    const val SETTINGS_EMAIL = "settings/email"
    const val SETTINGS_BUG = "settings/bug"
    const val SETTINGS_SUPPORT = "settings/support"
    const val SETTINGS_PRIVACY = "settings/privacy"
    const val SETTINGS_TERMS = "settings/terms"
    const val SETTINGS_ABOUT = "settings/about"
    const val SHOP = "shop"
    const val INVENTORY = "inventory"

    fun board(matchId: String, color: String) = "board/$matchId/$color"
}
