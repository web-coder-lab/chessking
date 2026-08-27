package com.geniusclan.app.ui.navigation

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import com.geniusclan.app.data.api.ApiClient
import com.geniusclan.app.data.api.TokenStore
import com.geniusclan.app.ui.screens.auth.AuthScreen
import com.geniusclan.app.ui.screens.board.BoardScreen
import com.geniusclan.app.ui.screens.home.HomeScreen
import com.geniusclan.app.ui.screens.play.PlayScreen
import com.geniusclan.app.ui.screens.profile.ProfileScreen
import com.geniusclan.app.ui.screens.splash.ServerGateScreen
import com.geniusclan.app.ui.screens.splash.SplashScreen
import com.geniusclan.app.ui.screens.wallet.WalletScreen

@Composable
fun AppNav() {
    val nav = rememberNavController()
    val context = LocalContext.current
    var sessionReady by remember { mutableStateOf(false) }
    var hasSession by remember { mutableStateOf(false) }

    LaunchedEffect(Unit) {
        hasSession = TokenStore.hasSession(context)
        sessionReady = true
    }

    if (!sessionReady) return

    NavHost(navController = nav, startDestination = Routes.SPLASH) {
        composable(Routes.SPLASH) {
            SplashScreen(
                onFinished = {
                    nav.navigate(Routes.SERVER_GATE) {
                        popUpTo(Routes.SPLASH) { inclusive = true }
                    }
                }
            )
        }
        composable(Routes.SERVER_GATE) {
            ServerGateScreen(
                onReady = {
                    val dest = if (hasSession) Routes.HOME else Routes.AUTH
                    nav.navigate(dest) {
                        popUpTo(Routes.SERVER_GATE) { inclusive = true }
                    }
                }
            )
        }
        composable(Routes.AUTH) {
            AuthScreen(
                onLoggedIn = {
                    TokenStore.save(context, ApiClient.accessToken, ApiClient.refreshToken)
                    hasSession = true
                    nav.navigate(Routes.HOME) {
                        popUpTo(Routes.AUTH) { inclusive = true }
                    }
                }
            )
        }
        composable(Routes.HOME) {
            HomeScreen(
                onPlay = { nav.navigate(Routes.PLAY) },
                onWallet = { nav.navigate(Routes.WALLET) },
                onProfile = { nav.navigate(Routes.PROFILE) }
            )
        }
        composable(Routes.PLAY) {
            PlayScreen(
                onBack = { nav.popBackStack() },
                onMatchFound = { matchId, color ->
                    nav.navigate(Routes.board(matchId, color)) {
                        popUpTo(Routes.PLAY) { inclusive = true }
                    }
                }
            )
        }
        composable(
            route = Routes.BOARD,
            arguments = listOf(
                navArgument("matchId") { type = NavType.StringType },
                navArgument("color") { type = NavType.StringType }
            )
        ) { entry ->
            val matchId = entry.arguments?.getString("matchId").orEmpty()
            val color = entry.arguments?.getString("color") ?: "white"
            BoardScreen(
                matchId = matchId,
                myColor = color,
                onLeave = {
                    nav.navigate(Routes.HOME) {
                        popUpTo(Routes.HOME) { inclusive = true }
                    }
                }
            )
        }
        composable(Routes.WALLET) {
            WalletScreen(onBack = { nav.popBackStack() })
        }
        composable(Routes.PROFILE) {
            ProfileScreen(
                onBack = { nav.popBackStack() },
                onLogout = {
                    TokenStore.clear(context)
                    hasSession = false
                    nav.navigate(Routes.AUTH) {
                        popUpTo(0) { inclusive = true }
                    }
                }
            )
        }
    }
}
