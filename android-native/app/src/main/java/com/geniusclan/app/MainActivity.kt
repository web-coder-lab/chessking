package com.geniusclan.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import com.geniusclan.app.ui.navigation.AppNav
import com.geniusclan.app.ui.theme.GcBg
import com.geniusclan.app.ui.theme.GeniusClanTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            GeniusClanTheme {
                Surface(modifier = Modifier.fillMaxSize(), color = GcBg) {
                    AppNav()
                }
            }
        }
    }
}
