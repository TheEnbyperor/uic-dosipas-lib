const SECP256_R1: rasn::types::ObjectIdentifier = rasn::types::ObjectIdentifier::new_unchecked(
    std::borrow::Cow::Borrowed(&[1, 2, 840, 10045, 3, 1, 7]));
const ECDSA_SHA256: rasn::types::ObjectIdentifier = rasn::types::ObjectIdentifier::new_unchecked(
    std::borrow::Cow::Borrowed(&[1, 2, 840, 10045, 4, 3, 2]));

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PublicKey {
    P256(p256::ecdsa::VerifyingKey),
}

impl PublicKey {
    pub(crate) fn load_level2_pubkey(alg: &Option<rasn::types::ObjectIdentifier>, pk: &[u8]) -> Result<Self, crate::BarcodeError> {
        let Some(alg) = alg else {
            return Err(crate::BarcodeError::UnsupportedData{ msg: "No level 2 public key algorithm provided".into() });
        };
        if alg == &SECP256_R1 {
            Ok(PublicKey::P256(p256::ecdsa::VerifyingKey::from_sec1_bytes(pk)
                .map_err(|e| crate::BarcodeError::CryptographyError{ msg: e.to_string() })?))
        } else {
            Err(crate::BarcodeError::UnsupportedData{ msg: format!("Unsupported level 2 public key: {}", alg) })
        }
    }

    pub fn public_bytes(&self) -> Vec<u8> {
        match self {
            Self::P256(key) => key.to_sec1_bytes().to_vec(),
        }
    }

    pub fn eq_private_key(&self, other: &PrivateKey) -> bool {
        self == other
    }

    pub fn key_alg_str(&self) -> String {
        self.key_alg().to_string()
    }

    pub fn key_alg_list(&self) -> Vec<u32> {
        self.key_alg().to_vec()
    }

    pub fn signing_alg_str(&self) -> String {
        self.signing_alg().to_string()
    }

    pub fn signing_alg_list(&self) -> Vec<u32> {
        self.signing_alg().to_vec()
    }

    pub(crate) fn key_alg(&self) -> rasn::types::ObjectIdentifier {
        match self {
            Self::P256(_) => SECP256_R1,
        }
    }

    pub(crate) fn signing_alg(&self) -> rasn::types::ObjectIdentifier {
        match self {
            Self::P256(_) => ECDSA_SHA256,
        }
    }
}

impl PartialEq<PrivateKey> for PublicKey {
    fn eq(&self, other: &PrivateKey) -> bool {
        *self == *other.public_key()
    }
}

enum PrivateKeyVariant {
    P256
}

pub struct PrivateKey {
    variant: PrivateKeyVariant,
    key: std::sync::Arc<dyn crate::KeychainKey>
}

impl PrivateKey {
    pub(crate) fn gen_p256(keychain: std::sync::Arc<dyn crate::Keychain>) -> Self {
        let key_ref = keychain.gen_p256_key();
        Self {
            variant: PrivateKeyVariant::P256,
            key: key_ref
        }
    }

    pub(crate) fn from_keychain(keychain: std::sync::Arc<dyn crate::Keychain>, key_id: &str) -> Result<Self, crate::BarcodeError> {
        let key_ref = keychain.get_by_key_id(key_id.to_string())
            .ok_or_else(|| crate::BarcodeError::CryptographyError { msg: format!("key ID not found: {}", key_id) })?;
        Ok(Self {
            variant: match key_ref.variant() {
                crate::KeyVariant::P256 => PrivateKeyVariant::P256,
            },
            key: key_ref
        })
    }

    pub fn key_id(&self) -> String {
        self.key.key_id()
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.key.sign(data.to_vec())
    }

    pub fn public_key(&self) -> std::sync::Arc<PublicKey> {
        match &self.variant {
            PrivateKeyVariant::P256 => {
                std::sync::Arc::new(PublicKey::P256(p256::ecdsa::VerifyingKey::from_sec1_bytes(&self.key.public_key()).unwrap()))
            }
        }
    }
}