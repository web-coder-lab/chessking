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

    fun board(matchId: String, color: String) = "board/$matchId/$color"
}
