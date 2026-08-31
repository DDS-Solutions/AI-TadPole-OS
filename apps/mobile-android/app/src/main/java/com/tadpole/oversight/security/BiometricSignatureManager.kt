/**
 * @docs ARCHITECTURE:Security
 * ### AI Assist Note
 * Android companion Ed25519 identity and biometric decision proof manager.
 * ### 🔍 Debugging & Observability
 * Failure Path: AndroidKeyStore unwrap failure or biometric signing denial.
 * Telemetry Link: Search `Companion signing` in Android logs.
 */
package com.tadpole.oversight.security

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import org.bouncycastle.jce.provider.BouncyCastleProvider
import java.security.KeyFactory
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.PrivateKey
import java.security.SecureRandom
import java.security.Signature
import java.security.spec.PKCS8EncodedKeySpec
import java.util.Base64
import java.util.UUID
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

data class SignedDecisionProof(
    val timestamp: Long,
    val nonce: String,
    val signature: String
)

/**
 * Owns the companion Ed25519 identity. Private bytes are encrypted at rest by
 * an AES-GCM key held in AndroidKeyStore; signing failures are fail-closed.
 */
class BiometricSignatureManager(private val context: Context) {

    private val identityPreferences = context.getSharedPreferences(
        "tadpole_companion_identity",
        Context.MODE_PRIVATE
    )
    private val provider = BouncyCastleProvider()
    private val wrappingKeyAlias = "TadpoleOS_Companion_Identity_Wrapping_Key"

    fun canAuthenticate(): Boolean {
        val biometricManager = BiometricManager.from(context)
        val authenticators = BiometricManager.Authenticators.BIOMETRIC_STRONG or
                BiometricManager.Authenticators.DEVICE_CREDENTIAL
        return biometricManager.canAuthenticate(authenticators) == BiometricManager.BIOMETRIC_SUCCESS
    }

    fun getPublicKeyHex(): String {
        ensureIdentity()
        val encoded = Base64.getDecoder().decode(
            identityPreferences.getString(PUBLIC_KEY_PREF, null)
                ?: error("Companion public key is unavailable")
        )
        require(encoded.size >= ED25519_PUBLIC_KEY_SIZE) { "Invalid stored Ed25519 public key" }
        return "ed25519:" + encoded.takeLast(ED25519_PUBLIC_KEY_SIZE).joinToString("") {
            "%02x".format(it.toInt() and 0xff)
        }
    }

    fun signPayload(payload: String): String {
        ensureIdentity()
        val encryptedPrivateKey = identityPreferences.getString(PRIVATE_KEY_PREF, null)
            ?: error("Companion private key is unavailable")
        val privateBytes = decryptPrivateKey(Base64.getDecoder().decode(encryptedPrivateKey))
        val privateKey: PrivateKey = KeyFactory.getInstance("Ed25519", provider)
            .generatePrivate(PKCS8EncodedKeySpec(privateBytes))
        val signature = Signature.getInstance("Ed25519", provider).apply {
            initSign(privateKey)
            update(payload.toByteArray(Charsets.UTF_8))
        }
        return signature.sign().joinToString("") { "%02x".format(it.toInt() and 0xff) }
    }

    fun promptBiometricDecision(
        activity: FragmentActivity,
        approvalId: String,
        decision: String,
        onSuccess: (proof: SignedDecisionProof) -> Unit,
        onError: (errorMsg: String) -> Unit
    ) {
        val executor = ContextCompat.getMainExecutor(activity)
        val callback = object : BiometricPrompt.AuthenticationCallback() {
            override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                super.onAuthenticationSucceeded(result)
                try {
                    val timestamp = System.currentTimeMillis() / 1000
                    val nonce = UUID.randomUUID().toString().replace("-", "")
                    val canonical = "$approvalId:$decision:$timestamp:$nonce"
                    onSuccess(SignedDecisionProof(timestamp, nonce, signPayload(canonical)))
                } catch (error: Exception) {
                    onError(error.message ?: "Companion signing failed")
                }
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
            .setTitle("Confirm Oversight Decision")
            .setSubtitle("${decision.replaceFirstChar { it.uppercase() }} approval $approvalId")
            .setAllowedAuthenticators(
                BiometricManager.Authenticators.BIOMETRIC_STRONG or
                        BiometricManager.Authenticators.DEVICE_CREDENTIAL
            )
            .build()
        BiometricPrompt(activity, executor, callback).authenticate(promptInfo)
    }

    private fun ensureIdentity() {
        if (identityPreferences.contains(PUBLIC_KEY_PREF) &&
            identityPreferences.contains(PRIVATE_KEY_PREF)
        ) return

        val keyPair = KeyPairGenerator.getInstance("Ed25519", provider).generateKeyPair()
        val encryptedPrivateKey = encryptPrivateKey(keyPair.private.encoded)
        val saved = identityPreferences.edit()
            .putString(PUBLIC_KEY_PREF, Base64.getEncoder().encodeToString(keyPair.public.encoded))
            .putString(PRIVATE_KEY_PREF, Base64.getEncoder().encodeToString(encryptedPrivateKey))
            .commit()
        check(saved) { "Unable to persist companion signing identity" }
    }

    private fun getOrCreateWrappingKey(): SecretKey {
        val keyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        (keyStore.getKey(wrappingKeyAlias, null) as? SecretKey)?.let { return it }

        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
        generator.init(
            KeyGenParameterSpec.Builder(
                wrappingKeyAlias,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
            )
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .build()
        )
        return generator.generateKey()
    }

    private fun encryptPrivateKey(privateBytes: ByteArray): ByteArray {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, getOrCreateWrappingKey(), SecureRandom())
        return cipher.iv + cipher.doFinal(privateBytes)
    }

    private fun decryptPrivateKey(encrypted: ByteArray): ByteArray {
        require(encrypted.size > GCM_IV_SIZE) { "Invalid encrypted companion key" }
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(
            Cipher.DECRYPT_MODE,
            getOrCreateWrappingKey(),
            GCMParameterSpec(128, encrypted.copyOfRange(0, GCM_IV_SIZE))
        )
        return cipher.doFinal(encrypted.copyOfRange(GCM_IV_SIZE, encrypted.size))
    }

    private companion object {
        const val PUBLIC_KEY_PREF = "ed25519_public_key"
        const val PRIVATE_KEY_PREF = "encrypted_ed25519_private_key"
        const val ED25519_PUBLIC_KEY_SIZE = 32
        const val GCM_IV_SIZE = 12
    }
}
