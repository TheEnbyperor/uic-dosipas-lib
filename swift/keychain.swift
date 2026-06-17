import Foundation
import Security

func randomString(length: Int) -> String {
  let letters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
  return String((0..<length).map{ _ in letters.randomElement()! })
}

public final class AppleKeychainKey: KeychainKey {
    let privateKey: SecKey
    let id: String
    
    init(privateKey: SecKey, keyId: String) {
        self.privateKey = privateKey
        self.id = keyId
    }
    
    public func keyId() -> String {
        return self.id
    }
    
    public func variant() -> KeyVariant {
        let attr = SecKeyCopyAttributes(self.privateKey)! as! [String: AnyObject]
        if attr[kSecAttrKeyType as String] as! CFString == kSecAttrKeyTypeECSECPrimeRandom && attr[kSecAttrKeySizeInBits as String] as! Int == 256 {
            return KeyVariant.p256
        } else {
            fatalError("Unexpected key variant")
        }
    }
    
    public func publicKey() -> Data {
        let publicKey = SecKeyCopyPublicKey(self.privateKey)!
        let publicKeyBytes = SecKeyCopyExternalRepresentation(publicKey, nil)!
        return publicKeyBytes as Data
    }
    
    public func sign(message: Data) -> Data {
        let algorithm: SecKeyAlgorithm = switch (self.variant()) {
        case .p256:
            if #available(macOS 14.0, *) {
                .ecdsaSignatureMessageRFC4754SHA256
            } else {
                .ecdsaSignatureRFC4754
            }
        }
        return SecKeyCreateSignature(self.privateKey, algorithm, message as CFData, nil)! as Data
    }
}

public final class AppleKeychain: Keychain {
    public init() {}

    public func genP256Key() -> any KeychainKey {
        let keyId = randomString(length: 32)
        let accessControl = SecAccessControlCreateWithFlags(
            kCFAllocatorDefault,
            kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
            .privateKeyUsage,
            nil
        )!
        let attributes: NSDictionary = [
            kSecAttrKeyType: kSecAttrKeyTypeECSECPrimeRandom,
            kSecAttrKeySizeInBits: 256,
            kSecAttrTokenID: kSecAttrTokenIDSecureEnclave,
            kSecPrivateKeyAttrs: [
                kSecAttrIsPermanent: true,
                kSecAttrApplicationTag: "org.uic.dosipas.deviceKey.\(keyId)",
                kSecAttrAccessControl: accessControl
            ]
        ]
        let privateKey = SecKeyCreateRandomKey(attributes as CFDictionary, nil)!
        return AppleKeychainKey(privateKey: privateKey, keyId: keyId)
    }
    
    public func getByKeyId(keyId: String) -> (any KeychainKey)? {
        let getquery: [String: Any] = [
            kSecClass as String: kSecClassKey,
            kSecAttrApplicationTag as String: "org.uic.dosipas.deviceKey.\(keyId)",
            kSecReturnRef as String: true
        ]
        var item: CFTypeRef?
        let status = SecItemCopyMatching(getquery as CFDictionary, &item)
        guard status == errSecSuccess else { return nil }
        let privateKey = item as! SecKey
        return AppleKeychainKey(privateKey: privateKey, keyId: keyId)
    }
}
