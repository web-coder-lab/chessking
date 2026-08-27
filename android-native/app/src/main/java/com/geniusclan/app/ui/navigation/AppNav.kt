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
import com.geniusclan.app.ui.screens.wallet.CheckoutScreen
import com.geniusclan.app.ui.screens.wallet.HistoryScreen
import com.geniusclan.app.ui.screens.settings.SettingsHomeScreen
import com.geniusclan.app.ui.screens.settings.TwoFactorScreen
import com.geniusclan.app.ui.screens.settings.SessionsScreen
import com.geniusclan.app.ui.screens.settings.ChangePasswordScreen
import com.geniusclan.app.ui.screens.settings.ChangeEmailScreen
import com.geniusclan.app.ui.screens.settings.LegalScreen
import com.geniusclan.app.ui.screens.settings.LegalKind
import com.geniusclan.app.ui.screens.settings.BugReportScreen
import com.geniusclan.app.ui.screens.shop.ShopScreen
import com.geniusclan.app.ui.screens.shop.InventoryScreen
import com.geniusclan.app.ui.screens.hub.DashboardScreen
import com.geniusclan.app.ui.screens.hub.LeaderboardScreen
import com.geniusclan.app.ui.screens.hub.NotificationsScreen
import com.geniusclan.app.ui.screens.hub.InviteScreen
import com.geniusclan.app.ui.screens.custom.CustomMatchScreen
import com.geniusclan.app.ui.screens.custom.MatchHistoryScreen

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
                onProfile = { nav.navigate(Routes.PROFILE) },
                onSettings = { nav.navigate(Routes.SETTINGS) },
                onShop = { nav.navigate(Routes.SHOP) },
                onInventory = { nav.navigate(Routes.INVENTORY) },
                onDashboard = { nav.navigate(Routes.DASHBOARD) },
                onLeaderboard = { nav.navigate(Routes.LEADERBOARD) },
                onNotifications = { nav.navigate(Routes.NOTIFICATIONS) },
                onInvite = { nav.navigate(Routes.INVITE) },
                onCustomMatch = { nav.navigate(Routes.CUSTOM_MATCH) },
                onMatchHistory = { nav.navigate(Routes.MATCH_HISTORY) }
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
            WalletScreen(
                onBack = { nav.popBackStack() },
                onAddCoins = { nav.navigate(Routes.CHECKOUT) },
                onHistory = { nav.navigate(Routes.WALLET_HISTORY) }
            )
        }
        composable(Routes.CHECKOUT) {
            CheckoutScreen(onBack = { nav.popBackStack() })
        }
        composable(Routes.WALLET_HISTORY) {
            HistoryScreen(onBack = { nav.popBackStack() })
        }
        composable(Routes.PROFILE) {
            ProfileScreen(
                onBack = { nav.popBackStack() },
                onSettings = { nav.navigate(Routes.SETTINGS) },
                onLogout = {
                    TokenStore.clear(context)
                    hasSession = false
                    nav.navigate(Routes.AUTH) {
                        popUpTo(0) { inclusive = true }
                    }
                }
            )
        }
        composable(Routes.SETTINGS) {
            SettingsHomeScreen(
                onBack = { nav.popBackStack() },
                onOpen = { key ->
                    val route = when (key) {
                        "2fa" -> Routes.SETTINGS_2FA
                        "sessions" -> Routes.SETTINGS_SESSIONS
                        "password" -> Routes.SETTINGS_PASSWORD
                        "email" -> Routes.SETTINGS_EMAIL
                        "bug" -> Routes.SETTINGS_BUG
                        "support" -> Routes.SETTINGS_SUPPORT
                        "privacy" -> Routes.SETTINGS_PRIVACY
                        "terms" -> Routes.SETTINGS_TERMS
                        "about" -> Routes.SETTINGS_ABOUT
                        else -> Routes.SETTINGS
                    }
                    nav.navigate(route)
                }
            )
        }
        composable(Routes.SETTINGS_2FA) {
            TwoFactorScreen(onBack = { nav.popBackStack() })
        }
        composable(Routes.SETTINGS_SESSIONS) {
            SessionsScreen(onBack = { nav.popBackStack() })
        }
        composable(Routes.SETTINGS_PASSWORD) {
            ChangePasswordScreen(onBack = { nav.popBackStack() })
        }
        composable(Routes.SETTINGS_EMAIL) {
            ChangeEmailScreen(onBack = { nav.popBackStack() })
        }
        composable(Routes.SETTINGS_BUG) {
            BugReportScreen(onBack = { nav.popBackStack() })
        }
        composable(Routes.SETTINGS_SUPPORT) {
            LegalScreen(LegalKind.SUPPORT, onBack = { nav.popBackStack() })
        }
        composable(Routes.SETTINGS_PRIVACY) {
            LegalScreen(LegalKind.PRIVACY, onBack = { nav.popBackStack() })
        }
        composable(Routes.SETTINGS_TERMS) {
            LegalScreen(LegalKind.TERMS, onBack = { nav.popBackStack() })
        }
        composable(Routes.SETTINGS_ABOUT) {
            LegalScreen(LegalKind.ABOUT, onBack = { nav.popBackStack() })
        }
        composable(Routes.SHOP) {
            ShopScreen(
                onBack = { nav.popBackStack() },
                onOpenInventory = { nav.navigate(Routes.INVENTORY) }
            )
        }
        composable(Routes.INVENTORY) {
            InventoryScreen(
                onBack = { nav.popBackStack() },
                onOpenShop = { nav.navigate(Routes.SHOP) }
            )
        }
        composable(Routes.DASHBOARD) {
            DashboardScreen(
                onBack = { nav.popBackStack() },
                onPlay = { nav.navigate(Routes.PLAY) },
                onLeaderboard = { nav.navigate(Routes.LEADERBOARD) },
                onNotifications = { nav.navigate(Routes.NOTIFICATIONS) },
                onInvite = { nav.navigate(Routes.INVITE) },
                onCustomMatch = { nav.navigate(Routes.CUSTOM_MATCH) },
                onMatchHistory = { nav.navigate(Routes.MATCH_HISTORY) }
            )
        }
        composable(Routes.LEADERBOARD) {
            LeaderboardScreen(onBack = { nav.popBackStack() })
        }
        composable(Routes.NOTIFICATIONS) {
            NotificationsScreen(onBack = { nav.popBackStack() })
        }
        composable(Routes.INVITE) {
            InviteScreen(onBack = { nav.popBackStack() })
        }
        composable(Routes.CUSTOM_MATCH) {
            CustomMatchScreen(
                onBack = { nav.popBackStack() },
                onMatchReady = { matchId, color ->
                    nav.navigate(Routes.board(matchId, color))
                }
            )
        }
        composable(Routes.MATCH_HISTORY) {
            MatchHistoryScreen(onBack = { nav.popBackStack() })
        }
    }
}
