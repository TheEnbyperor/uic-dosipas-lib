package org.uic.dosipas
import java.math.BigInteger
import java.security.AlgorithmParameters
import java.security.PrivateKey
import java.security.PublicKey
import java.security.Signature
import java.security.interfaces.ECPublicKey
import java.security.spec.ECFieldFp
import java.security.spec.ECGenParameterSpec
import java.security.spec.ECParameterSpec
import java.security.spec.ECPoint


private val P256_SPEC: ECParameterSpec by lazy {
    AlgorithmParameters.getInstance("EC").apply {
        init(ECGenParameterSpec("secp256r1"))
    }.getParameterSpec(ECParameterSpec::class.java)
}

private fun ECParameterSpec.isP256(): Boolean {
    val want = P256_SPEC
    val field = curve.field as? ECFieldFp ?: return false
    val wantField = want.curve.field as? ECFieldFp ?: return false

    return field.p == wantField.p &&
            curve.a == want.curve.a &&
            curve.b == want.curve.b &&
            generator == want.generator &&
            order == want.order &&
            cofactor == want.cofactor
}

class AndroidKeychainKey(val keyId: String, val priv: PrivateKey, val pub: PublicKey) : KeychainKey {
    override fun keyId(): String {
        return keyId
    }

    fun toSec1Compressed(point: ECPoint, fieldSizeBytes: Int): ByteArray {
        val x = point.affineX
        val y = point.affineY
        val prefix = if (y.testBit(0)) 0x03.toByte() else 0x02.toByte()
        val xb = toFixedLength(x, fieldSizeBytes / 8)
        val out = ByteArray(1 + xb.size)
        out[0] = prefix
        System.arraycopy(xb, 0, out, 1, xb.size)
        return out
    }

    private fun toFixedLength(v: BigInteger, len: Int): ByteArray {
        val bytes = v.toByteArray()

        return when {
            bytes.size == len -> bytes
            bytes.size == len + 1 && bytes[0] == 0.toByte() ->
                bytes.copyOfRange(1, bytes.size)
            bytes.size < len ->
                ByteArray(len - bytes.size) + bytes
            else ->
                throw IllegalArgumentException("Integer too large")
        }
    }

    private fun derToRawEcdsaP256(der: ByteArray): ByteArray {
        var i = 0
        fun readByte(): Int {
            if (i >= der.size) throw IllegalArgumentException("Truncated ECDSA signature")
            return der[i++].toInt() and 0xff
        }

        fun readLen(): Int {
            val first = readByte()
            return if (first and 0x80 == 0) {
                first
            } else {
                val n = first and 0x7f
                if (n == 0 || n > 4) throw IllegalArgumentException("Invalid DER length")
                var len = 0
                repeat(n) { len = (len shl 8) or readByte() }
                len
            }
        }

        fun readInteger32(): ByteArray {
            if (readByte() != 0x02) throw IllegalArgumentException("Expected DER INTEGER")
            val len = readLen()
            if (len <= 0 || i + len > der.size) throw IllegalArgumentException("Truncated INTEGER")
            var start = i
            val end = i + len
            if (der[start] == 0.toByte()) start++
            val magLen = end - start
            if (magLen > 32) throw IllegalArgumentException("P-256 INTEGER too large")
            val out = ByteArray(32)
            System.arraycopy(der, start, out, 32 - magLen, magLen)
            i = end
            return out
        }

        if (readByte() != 0x30) throw IllegalArgumentException("Expected DER SEQUENCE")
        val seqLen = readLen()
        if (seqLen != der.size - i) throw IllegalArgumentException("Invalid DER length")
        val r = readInteger32()
        val s = readInteger32()
        if (i != der.size) throw IllegalArgumentException("Trailing data in signature")
        return r + s
    }

    override fun publicKey(): ByteArray {
        val ecPub = pub as? ECPublicKey
            ?: throw NotImplementedError("Only EC public keys are supported")
        return toSec1Compressed(ecPub.w, ecPub.params.curve.field.fieldSize)
    }

    override fun sign(message: ByteArray): ByteArray {
        val sig = Signature.getInstance("SHA256withECDSA")
        sig.initSign(priv)
        sig.update(message)
        return derToRawEcdsaP256(sig.sign())
    }

    override fun variant(): KeyVariant {
        val ecPub = pub as? ECPublicKey
            ?: throw NotImplementedError("Only EC public keys are supported")
        if (ecPub.params.isP256()) {
            return KeyVariant.P256
        } else {
            throw NotImplementedError("Unknown EC params")
        }
    }
}