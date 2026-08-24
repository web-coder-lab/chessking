package com.geniusclan.app.ui.navigation

object Routes {
    const val SPLASH = "splash"
    const val SERVER_GATE = "server_gate"
    const val AUTH = "auth"
    const val HOME = "home"
    const val PLAY = "play"
    const val WALLET = "wallet"
    const val SHOP = "shop"
    const val PROFILE = "profile"
    const val SETTINGS = "settings"
    const val BOARD = "board/{matchId}"

    fun board(matchId: String) = "board/$matchId"
}
