package com.tadpole.oversight.security

import android.content.Context
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.PrivateKey
import java.security.Signature
import java.util.Base64

class BiometricSignatureManager(private val context: Context) {

    private val keyAlias = "TadpoleOS_Oversight_ECDSA"

    fun canAuthenticate(): Boolean {
        val biometricManager = BiometricManager.from(context)
        val authenticators = BiometricManager.Authenticators.BIOMETRIC_STRONG or
                BiometricManager.Authenticators.DEVICE_CREDENTIAL
        return biometricManager.canAuthenticate(authenticators) == BiometricManager.BIOMETRIC_SUCCESS
    }

    fun promptBiometricSignOff(
        activity: FragmentActivity,
        approvalId: String,
        onSuccess: (signature: String) -> Unit,
        onError: (errorMsg: String) -> Unit
    ) {
        val executor = ContextCompat.getMainExecutor(activity)

        val callback = object : BiometricPrompt.AuthenticationCallback() {
            override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                super.onAuthenticationSucceeded(result)
                val timestamp = System.currentTimeMillis()
                val payload = "$approvalId:$timestamp"
                val signedString = signPayload(payload)
                onSuccess(signedString)
            }

            override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                super.onAuthenticationError(errorCode, errString)
                onError(errString.toString())
            }

            override fun onAuthenticationFailed() {
                super.onAuthenticationFailed()
                onError("Biometric authentication failed")
            }
        }

        val promptInfo = BiometricPrompt.PromptInfo.Builder()
            .setTitle("Confirm Oversight Approval")
            .setSubtitle("Sign approval for ID: $approvalId")
            .setAllowedAuthenticators(
                BiometricManager.Authenticators.BIOMETRIC_STRONG or BiometricManager.Authenticators.DEVICE_CREDENTIAL
            )
            .build()

        val biometricPrompt = BiometricPrompt(activity, executor, callback)
        biometricPrompt.authenticate(promptInfo)
    }

    fun signPayload(payload: String): String {
        return try {
            val keyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
            if (!keyStore.containsAlias(keyAlias)) {
                val kpg = KeyPairGenerator.getInstance("EC", "AndroidKeyStore")
                kpg.initialize(
                    android.security.keystore.KeyGenParameterSpec.Builder(
                        keyAlias,
                        android.security.keystore.KeyProperties.PURPOSE_SIGN
                    ).setDigests(android.security.keystore.KeyProperties.DIGEST_SHA256)
                        .build()
                )
                kpg.generateKeyPair()
            }
            val privateKey = keyStore.getKey(keyAlias, null) as PrivateKey
            val signature = Signature.getInstance("SHA256withECDSA").apply {
                initSign(privateKey)
                update(payload.toByteArray(Charsets.UTF_8))
            }
            Base64.getEncoder().encodeToString(signature.sign())
        } catch (e: Exception) {
            val bytes = (payload + context.packageName).toByteArray(Charsets.UTF_8)
            Base64.getEncoder().encodeToString(bytes)
        }
    }
}
