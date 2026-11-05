/**
 * @docs ARCHITECTURE:Configuration
 * ### AI Assist Note
 * Persists the companion endpoint, pairing state, and stable device identifier.
 * ### 🔍 Debugging & Observability
 * Failure Path: SharedPreferences identity persistence failure.
 * Telemetry Link: Search `tadpole_settings` in Android diagnostics.
 */
package com.tadpole.oversight.data.settings

import android.content.Context
import android.content.SharedPreferences
import java.util.UUID

class SettingsRepository(context: Context) {
    private val prefs: SharedPreferences = context.getSharedPreferences("tadpole_settings", Context.MODE_PRIVATE)

    fun getNodeIp(): String {
        return prefs.getString("node_ip", "10.0.0.1:8000") ?: "10.0.0.1:8000"
    }

    fun setNodeIp(ip: String) {
        prefs.edit().putString("node_ip", ip).apply()
    }

    fun getPairingToken(): String? {
        return prefs.getString("pairing_token", null)
    }

    fun setPairingToken(token: String) {
        prefs.edit().putString("pairing_token", token).apply()
    }

    fun isPaired(): Boolean {
        return prefs.getBoolean("is_paired", false)
    }

    fun setPaired(paired: Boolean) {
        prefs.edit().putBoolean("is_paired", paired).apply()
    }

    fun getOrCreateDeviceId(): String {
        prefs.getString("device_id", null)?.let { return it }
        val deviceId = "android-${UUID.randomUUID()}"
        check(prefs.edit().putString("device_id", deviceId).commit()) {
            "Unable to persist companion device identity"
        }
        return deviceId
    }
}
