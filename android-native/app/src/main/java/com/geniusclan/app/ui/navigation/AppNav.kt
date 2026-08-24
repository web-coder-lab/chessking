package com.geniusclan.app.ui.navigation

import androidx.compose.runtime.Composable
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.geniusclan.app.ui.screens.auth.AuthScreen
import com.geniusclan.app.ui.screens.home.HomeScreen
import com.geniusclan.app.ui.screens.splash.ServerGateScreen
import com.geniusclan.app.ui.screens.splash.SplashScreen

@Composable
fun AppNav() {
    val nav = rememberNavController()
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
                    nav.navigate(Routes.AUTH) {
                        popUpTo(Routes.SERVER_GATE) { inclusive = true }
                    }
                }
            )
        }
        composable(Routes.AUTH) {
            AuthScreen(
                onLoggedIn = {
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
            HomeScreen(
                title = "Play",
                subtitle = "Matchmaking UI — Phase 4",
                onPlay = {},
                onWallet = { nav.navigate(Routes.WALLET) },
                onProfile = { nav.navigate(Routes.PROFILE) }
            )
        }
        composable(Routes.WALLET) {
            HomeScreen(
                title = "Wallet",
                subtitle = "Coins & payments — Phase 3",
                onPlay = { nav.navigate(Routes.PLAY) },
                onWallet = {},
                onProfile = { nav.navigate(Routes.PROFILE) }
            )
        }
        composable(Routes.PROFILE) {
            HomeScreen(
                title = "Profile",
                subtitle = "Edit profile — Phase 3",
                onPlay = { nav.navigate(Routes.PLAY) },
                onWallet = { nav.navigate(Routes.WALLET) },
                onProfile = {}
            )
        }
    }
}
