package org.uic.dosipas

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.spec.ECGenParameterSpec


fun randomString(length: Int) : String {
    val allowedChars = ('A'..'Z') + ('a'..'z') + ('0'..'9')
    return (1..length)
        .map { allowedChars.random() }
        .joinToString("")
}

class AndroidKeychain: Keychain {
    val ks: KeyStore = KeyStore.getInstance("AndroidKeyStore").apply {
        load(null)
    }

    override fun genP256Key(): KeychainKey {
        val keyId = randomString(32)
        val kpg: KeyPairGenerator = KeyPairGenerator.getInstance(
            KeyProperties.KEY_ALGORITHM_EC,
            "AndroidKeyStore"
        )
        val parameterSpec: KeyGenParameterSpec = KeyGenParameterSpec.Builder(
            "org.uic.dosipas.deviceKey.${keyId}",
            KeyProperties.PURPOSE_SIGN
        ).run {
            setAlgorithmParameterSpec(ECGenParameterSpec("secp256r1"))
            setKeySize(256)
            setDigests(KeyProperties.DIGEST_SHA256)
            build()
        }
        kpg.initialize(parameterSpec)
        val kp = kpg.generateKeyPair()
        return AndroidKeychainKey(keyId, kp.private, kp.public)
    }

    override fun getByKeyId(keyId: String): KeychainKey? {
        val entry =
            ks.getEntry("org.uic.dosipas.deviceKey.${keyId}", null) as? KeyStore.PrivateKeyEntry
                ?: return null
        return AndroidKeychainKey(keyId, entry.privateKey, entry.certificate.publicKey)
    }
}

