package com.geniusclan.app.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

private val GeniusDarkScheme = darkColorScheme(
    primary = GcGold,
    onPrimary = GcBg,
    secondary = GcGoldDim,
    background = GcBg,
    onBackground = GcText,
    surface = GcSurface,
    onSurface = GcText,
    surfaceVariant = GcSurfaceHover,
    onSurfaceVariant = GcTextMuted,
    error = GcDanger,
    onError = Color.White,
    outline = GcBorder
)

@Composable
fun GeniusClanTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = GeniusDarkScheme,
        typography = GeniusTypography,
        content = content
    )
}
