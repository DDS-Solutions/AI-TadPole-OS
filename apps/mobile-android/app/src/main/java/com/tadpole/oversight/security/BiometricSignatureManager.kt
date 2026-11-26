/**
 * @docs ARCHITECTURE:Security
 * ### AI Assist Note
 * Android companion Ed25519 identity and hardware-keystore-bound biometric decision proof manager.
 * ### 🔍 Debugging & Observability
 * Failure Path: AndroidKeyStore unwrap failure, missing hardware CryptoObject, or biometric denial.
 * Telemetry Link: Search `Companion signing` in Android logs.
 */
package com.tadpole.oversight.security

import android.content.Context
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyPermanentlyInvalidatedException
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
import java.security.PublicKey
import java.security.SecureRandom
import java.security.Signature
import java.security.spec.MGF1ParameterSpec
import java.security.spec.PKCS8EncodedKeySpec
import java.util.Base64
import java.util.UUID
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
import javax.crypto.spec.OAEPParameterSpec
import javax.crypto.spec.PSource

data class SignedDecisionProof(
    val timestamp: Long,
    val nonce: String,
    val signature: String
)

/**
 * Owns the companion Ed25519 identity.
 * - Background request authentication (`signPayload`) uses an AndroidKeyStore AES-GCM wrapping key.
 * - Oversight decisions (`promptBiometricDecision`) wrap the Ed25519 key under an AndroidKeyStore
 *   RSA-2048 OAEP keypair configured with `setUserAuthenticationRequired(true)` and
 *   `setInvalidatedByBiometricEnrollment(true)`: the public key wraps at provisioning time, while
 *   unwrapping strictly requires a hardware-authenticated `BiometricPrompt.CryptoObject`.
 */
class BiometricSignatureManager(private val context: Context) {

    private val identityPreferences = context.getSharedPreferences(
        "tadpole_companion_identity",
        Context.MODE_PRIVATE
    )
    private val provider = BouncyCastleProvider()
    private val wrappingKeyAlias = "TadpoleOS_Companion_Identity_Wrapping_Key"
    private val biometricDecisionKeyAlias = "TadpoleOS_Companion_Biometric_Decision_RSA_Key"

    fun canAuthenticate(): Boolean {
        val biometricManager = BiometricManager.from(context)
        val authenticators = BiometricManager.Authenticators.BIOMETRIC_STRONG
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
        return signWithPrivateBytes(privateBytes, payload)
    }

    private fun signWithPrivateBytes(privateBytes: ByteArray, payload: String): String {
        try {
            val privateKey: PrivateKey = KeyFactory.getInstance("Ed25519", provider)
                .generatePrivate(PKCS8EncodedKeySpec(privateBytes))
            val signature = Signature.getInstance("Ed25519", provider).apply {
                initSign(privateKey)
                update(payload.toByteArray(Charsets.UTF_8))
            }
            return signature.sign().joinToString("") { "%02x".format(it.toInt() and 0xff) }
        } finally {
            privateBytes.fill(0)
        }
    }

    fun promptBiometricDecision(
        activity: FragmentActivity,
        approvalId: String,
        decision: String,
        onSuccess: (proof: SignedDecisionProof) -> Unit,
        onError: (errorMsg: String) -> Unit
    ) {
        val executor = ContextCompat.getMainExecutor(activity)
        val encryptedBiometricKey: ByteArray
        val cryptoObject: BiometricPrompt.CryptoObject
        try {
            ensureIdentity()
            val encodedBiometric = identityPreferences.getString(BIOMETRIC_PRIVATE_KEY_PREF, null)
                ?: error("Biometric-bound companion key is unavailable")
            encryptedBiometricKey = Base64.getDecoder().decode(encodedBiometric)

            val privateKey = getBiometricDecisionPrivateKey()
            val cipher = Cipher.getInstance(RSA_OAEP_TRANSFORMATION).apply {
                init(Cipher.DECRYPT_MODE, privateKey, OAEP_PARAMS)
            }
            cryptoObject = BiometricPrompt.CryptoObject(cipher)
        } catch (_: KeyPermanentlyInvalidatedException) {
            clearInvalidatedBiometricKey()
            onError(
                "Biometric enrollment changed and invalidated the hardware key. " +
                    "Please re-pair this companion device from Desktop Settings."
            )
            return
        } catch (error: Exception) {
            onError(error.message ?: "Failed to initialize hardware biometric CryptoObject")
            return
        }

        val callback = object : BiometricPrompt.AuthenticationCallback() {
            override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                super.onAuthenticationSucceeded(result)
                try {
                    val unlockedCipher = result.cryptoObject?.cipher
                        ?: error("Hardware-authenticated CryptoObject cipher missing")
                    val privateBytes = unlockedCipher.doFinal(encryptedBiometricKey)
                    val timestamp = System.currentTimeMillis() / 1000
                    val nonce = UUID.randomUUID().toString().replace("-", "")
                    val canonical = "$approvalId:$decision:$timestamp:$nonce"
                    val sigHex = signWithPrivateBytes(privateBytes, canonical)
                    onSuccess(SignedDecisionProof(timestamp, nonce, sigHex))
                } catch (error: Exception) {
                    onError(error.message ?: "Hardware-backed companion signing failed")
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
            .setNegativeButtonText("Cancel")
            .setAllowedAuthenticators(BiometricManager.Authenticators.BIOMETRIC_STRONG)
            .build()
        BiometricPrompt(activity, executor, callback).authenticate(promptInfo, cryptoObject)
    }

    private fun ensureIdentity() {
        val hasBase = identityPreferences.contains(PUBLIC_KEY_PREF) &&
            identityPreferences.contains(PRIVATE_KEY_PREF)
        val hasBiometric = identityPreferences.contains(BIOMETRIC_PRIVATE_KEY_PREF)
        if (hasBase && hasBiometric) return

        if (hasBase && !hasBiometric) {
            val existingEncrypted = identityPreferences.getString(PRIVATE_KEY_PREF, null)
                ?: error("Companion private key is unavailable")
            val rawPrivate = decryptPrivateKey(Base64.getDecoder().decode(existingEncrypted))
            try {
                val bioWrapped = wrapWithBiometricPublicKey(rawPrivate)
                val upgraded = identityPreferences.edit()
                    .putString(BIOMETRIC_PRIVATE_KEY_PREF, Base64.getEncoder().encodeToString(bioWrapped))
                    .commit()
                check(upgraded) { "Unable to persist biometric-bound companion signing identity" }
            } finally {
                rawPrivate.fill(0)
            }
            return
        }

        val keyPair = KeyPairGenerator.getInstance("Ed25519", provider).generateKeyPair()
        val rawPrivate = keyPair.private.encoded
        try {
            val encryptedPrivateKey = encryptWithAesKey(rawPrivate, getOrCreateWrappingKey())
            val biometricEncryptedPrivateKey = wrapWithBiometricPublicKey(rawPrivate)
            val saved = identityPreferences.edit()
                .putString(PUBLIC_KEY_PREF, Base64.getEncoder().encodeToString(keyPair.public.encoded))
                .putString(PRIVATE_KEY_PREF, Base64.getEncoder().encodeToString(encryptedPrivateKey))
                .putString(
                    BIOMETRIC_PRIVATE_KEY_PREF,
                    Base64.getEncoder().encodeToString(biometricEncryptedPrivateKey)
                )
                .commit()
            check(saved) { "Unable to persist companion signing identity" }
        } finally {
            rawPrivate.fill(0)
        }
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

    private fun clearInvalidatedBiometricKey() {
        runCatching {
            val keyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
            if (keyStore.containsAlias(biometricDecisionKeyAlias)) {
                keyStore.deleteEntry(biometricDecisionKeyAlias)
            }
        }
        identityPreferences.edit()
            .remove(BIOMETRIC_PRIVATE_KEY_PREF)
            .apply()
    }

    private fun ensureBiometricDecisionKeyPair(): KeyStore {
        val keyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        if (keyStore.containsAlias(biometricDecisionKeyAlias)) {
            return keyStore
        }

        val generator = KeyPairGenerator.getInstance(KeyProperties.KEY_ALGORITHM_RSA, "AndroidKeyStore")
        val builder = KeyGenParameterSpec.Builder(
            biometricDecisionKeyAlias,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
        )
            .setKeySize(2048)
            .setDigests(KeyProperties.DIGEST_SHA256, KeyProperties.DIGEST_SHA1)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_RSA_OAEP)
            .setUserAuthenticationRequired(true)
            .setInvalidatedByBiometricEnrollment(true)

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            builder.setUserAuthenticationParameters(
                0,
                KeyProperties.AUTH_BIOMETRIC_STRONG
            )
        } else {
            @Suppress("DEPRECATION")
            builder.setUserAuthenticationValidityDurationSeconds(-1)
        }

        generator.initialize(builder.build())
        generator.generateKeyPair()
        return keyStore
    }

    private fun getBiometricDecisionPublicKey(): PublicKey {
        val keyStore = ensureBiometricDecisionKeyPair()
        return keyStore.getCertificate(biometricDecisionKeyAlias)?.publicKey
            ?: error("Biometric decision public key is unavailable")
    }

    private fun getBiometricDecisionPrivateKey(): PrivateKey {
        val keyStore = ensureBiometricDecisionKeyPair()
        return keyStore.getKey(biometricDecisionKeyAlias, null) as? PrivateKey
            ?: error("Biometric decision private key is unavailable")
    }

    private fun wrapWithBiometricPublicKey(privateBytes: ByteArray): ByteArray {
        val cipher = Cipher.getInstance(RSA_OAEP_TRANSFORMATION)
        cipher.init(Cipher.ENCRYPT_MODE, getBiometricDecisionPublicKey(), OAEP_PARAMS)
        return cipher.doFinal(privateBytes)
    }

    private fun encryptWithAesKey(privateBytes: ByteArray, key: SecretKey): ByteArray {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, key, SecureRandom())
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
        const val BIOMETRIC_PRIVATE_KEY_PREF = "biometric_rsa_encrypted_ed25519_private_key"
        const val ED25519_PUBLIC_KEY_SIZE = 32
        const val GCM_IV_SIZE = 12
        const val RSA_OAEP_TRANSFORMATION = "RSA/ECB/OAEPWithSHA-256AndMGF1Padding"
        val OAEP_PARAMS = OAEPParameterSpec(
            "SHA-256",
            "MGF1",
            MGF1ParameterSpec.SHA1,
            PSource.PSpecified.DEFAULT
        )
    }
}
