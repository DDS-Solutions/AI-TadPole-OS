package com.tadpole.oversight.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

private val DarkColorScheme = darkColorScheme(
    primary = Color(0xFF10B981), // Sovereign Emerald Green
    secondary = Color(0xFF06B6D4),
    tertiary = Color(0xFFF59E0B),
    background = Color(0xFF090D16),
    surface = Color(0xFF111827),
    onPrimary = Color.White,
    onBackground = Color(0xFFF3F4F6),
    onSurface = Color(0xFFF3F4F6)
)

@Composable
fun TadpoleOSTheme(
    darkTheme: Boolean = true, // Default to sleek Sovereign dark theme
    content: @Composable () -> Unit
) {
    MaterialTheme(
        colorScheme = DarkColorScheme,
        content = content
    )
}
