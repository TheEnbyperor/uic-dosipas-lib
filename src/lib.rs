use thiserror::Error;

uniffi::include_scaffolding!("uic-dosipas");

mod asn1 {
    include!("../asn1_gen/barcode_header_v1_0_0.rs");
    include!("../asn1_gen/barcode_header_v2_0_1.rs");
    include!("../asn1_gen/dynamic_content_data_v1_0_5.rs");
}

mod sig;
mod dcd;

use sig::{PrivateKey, PublicKey};
use dcd::{DynamicContentData, GeoCoordinate, GeoCoordinateSystem, GeoUnit, LatLong};

#[uniffi::trait_interface]
trait Keychain: Send + Sync + std::fmt::Debug {
    fn gen_p256_key(&self) -> std::sync::Arc<dyn KeychainKey>;
    fn get_by_key_id(&self, key_id: String) -> Option<std::sync::Arc<dyn KeychainKey>>;
}

enum KeyVariant {
    P256
}

#[uniffi::trait_interface]
trait KeychainKey: Send + Sync + std::fmt::Debug {
    fn key_id(&self) -> String;
    fn variant(&self) -> KeyVariant;
    fn public_key(&self) -> Vec<u8>;
    fn sign(&self, data: Vec<u8>) -> Vec<u8>;
}

#[derive(Debug, Error)]
pub enum BarcodeError {
    #[error("the barcode format is not recognised")]
    UnknownBarcodeType,
    #[error("the data is invalid: `{msg}`")]
    InvalidFormat { msg: String },
    #[error("a cryptographic operation failed: `{msg}`")]
    CryptographyError { msg: String },
    #[error("a value was encountered which this library does not support: `{msg}`")]
    UnsupportedData { msg: String },
}

enum DosipasVersion {
    V1,
    V2,
}

struct Dosipas {
    pub security_provider: String,
    pub key_id: u32,
    pub end_of_validity: Option<chrono::DateTime<chrono::Utc>>,
    pub validity_duration: Option<chrono::TimeDelta>,
    pub records: Vec<Record>,
    pub level_2_data: std::sync::RwLock<Option<Record>>,
    pub(crate) version: DosipasVersion,
    pub(crate) level_1_data: Vec<u8>,
    pub(crate) level_1_signature: Vec<u8>,
    pub(crate) level_2_signing_alg: Option<rasn::types::ObjectIdentifier>,
    pub(crate) level_2_public_key: Option<sig::PublicKey>,
}

#[derive(Debug, Clone)]
pub struct Record {
    pub record_id: String,
    pub data: Vec<u8>,
}

impl Dosipas {
    fn new_from_bytes(data: &[u8]) -> Result<Dosipas, BarcodeError> {
        if let Ok(data) =
            rasn::uper::decode::<asn1::asn_module_header_v2::UicBarcodeHeader>(data)
        {
            Self::try_from(data)
        } else if let Ok(data) =
            rasn::uper::decode::<asn1::asn_module_header_v1::UicBarcodeHeader>(data)
        {
            Self::try_from(data)
        } else {
            Err(BarcodeError::UnknownBarcodeType)
        }
    }

    pub fn security_provider(&self) -> String {
        self.security_provider.clone()
    }

    pub fn key_id(&self) -> u32 {
        self.key_id
    }

    pub fn end_of_validity(&self) -> Option<std::time::SystemTime> {
        self.end_of_validity.map(|t| t.into())
    }

    pub fn validity_duration(&self) -> Option<std::time::Duration> {
        self.validity_duration.map(|t| std::time::Duration::from_secs_f64(t.as_seconds_f64()))
    }

    pub fn records(&self) -> Vec<Record> {
        self.records.clone()
    }

    pub fn level_2_data(&self) -> Option<Record> {
        self.level_2_data.read().unwrap().clone()
    }

    pub fn set_dynamic_content(&self, dynamic_content: &dcd::DynamicContentData) -> Result<(), BarcodeError> {
        let _ = self.level_2_data.write().unwrap().insert(Record {
            record_id: "FDC1".into(),
            data: rasn::uper::encode(&dynamic_content.to_asn1()?).unwrap()
        });
        Ok(())
    }

    pub fn is_matching_public_key(&self, public_key: &PublicKey) -> bool {
        let Some(defined_public_key) = &self.level_2_public_key else {
            return false;
        };
        let Some(defined_signing_alg) = &self.level_2_signing_alg else {
            return false;
        };
        *defined_public_key == *public_key && *defined_signing_alg == public_key.signing_alg()
    }

    pub fn sign(&self, private_key: &PrivateKey) -> Result<Vec<u8>, BarcodeError> {
        match self.version {
            DosipasVersion::V1 => {
                let level_1_data = asn1::asn_module_header_v1::Level2DataType {
                    level1_data: rasn::uper::decode(&self.level_1_data).unwrap(),
                    level1_signature: Some(self.level_1_signature.clone().into()),
                    level2_data: match self.level_2_data.read().unwrap().as_ref() {
                        Some(d) => Some(asn1::asn_module_header_v1::DataType {
                            data_format: rasn::types::Ia5String::from_iso646_bytes(d.record_id.as_bytes())
                                .map_err(|e| BarcodeError::InvalidFormat { msg: format!("Invalid level 2 record ID: {}", e) })?,
                            data: d.data.clone().into(),
                        }),
                        None => None,
                    }
                };
                let tbs_data = rasn::uper::encode(&level_1_data).unwrap();
                let sig = private_key.sign(&tbs_data);
                let level_2_data = asn1::asn_module_header_v1::UicBarcodeHeader {
                    format: rasn::types::Ia5String::from_iso646_bytes(b"U1").unwrap(),
                    level2_signed_data: level_1_data,
                    level2_signature: Some(sig.into()),
                };
                Ok(rasn::uper::encode(&level_2_data).unwrap())
            },
            DosipasVersion::V2 => {
                let level_1_data = asn1::asn_module_header_v2::Level2DataType {
                    level1_data: rasn::uper::decode(&self.level_1_data).unwrap(),
                    level1_signature: Some(self.level_1_signature.clone().into()),
                    level2_data: match self.level_2_data.read().unwrap().as_ref() {
                        Some(d) => Some(asn1::asn_module_header_v2::DataType {
                            data_format: rasn::types::Ia5String::from_iso646_bytes(d.record_id.as_bytes())
                                .map_err(|e| BarcodeError::InvalidFormat { msg: format!("Invalid level 2 record ID: {}", e) })?,
                            data: d.data.clone().into(),
                        }),
                        None => None,
                    }
                };
                let tbs_data = rasn::uper::encode(&level_1_data).unwrap();
                let sig = private_key.sign(&tbs_data);
                let level_2_data = asn1::asn_module_header_v2::UicBarcodeHeader {
                    format: rasn::types::Ia5String::from_iso646_bytes(b"U2").unwrap(),
                    level2_signed_data: level_1_data,
                    level2_signature: Some(sig.into()),
                };
                Ok(rasn::uper::encode(&level_2_data).unwrap())
            }
        }
    }
}

impl TryFrom<asn1::asn_module_header_v1::UicBarcodeHeader> for Dosipas {
    type Error = BarcodeError;
    fn try_from(header: asn1::asn_module_header_v1::UicBarcodeHeader) -> Result<Self, Self::Error> {
        if header.format.as_slice() != b"U1" {
            return Err(BarcodeError::UnknownBarcodeType);
        }

        let level_2_public_key = header.level2_signed_data.level1_data.level2_public_key.as_ref().map(|v| {
            PublicKey::load_level2_pubkey(&header.level2_signed_data.level1_data.level2_key_alg, &v)
        }).transpose()?;

        Ok(Self {
            version: DosipasVersion::V1,
            level_1_data: rasn::uper::encode(&header.level2_signed_data.level1_data).unwrap(),
            level_1_signature: header.level2_signed_data.level1_signature.unwrap_or_default().to_vec(),
            level_2_signing_alg: header.level2_signed_data.level1_data.level2_signing_alg,
            level_2_public_key,
            security_provider: if let Some(id) = header.level2_signed_data.level1_data.security_provider_num {
                id.to_string()
            } else if let Some(id) = &header.level2_signed_data.level1_data.security_provider_ia5 {
                id.to_string()
            } else {
                return Err(BarcodeError::InvalidFormat { msg: "One of securityProviderNum or securityProviderIA5 must be set".to_string() })
            },
            key_id: header.level2_signed_data.level1_data.key_id.unwrap_or_default(),
            end_of_validity: None,
            validity_duration: None,
            records: header.level2_signed_data.level1_data.data_sequence.into_iter().map(Into::into).collect::<Vec<_>>(),
            level_2_data: std::sync::RwLock::new(header.level2_signed_data.level2_data.map(Into::into)),
        })
    }
}

impl TryFrom<asn1::asn_module_header_v2::UicBarcodeHeader> for Dosipas {
    type Error = BarcodeError;
    fn try_from(header: asn1::asn_module_header_v2::UicBarcodeHeader) -> Result<Self, Self::Error> {
        if header.format.as_slice() != b"U2" {
            return Err(BarcodeError::UnknownBarcodeType);
        }

        let level_2_public_key = header.level2_signed_data.level1_data.level2_public_key.as_ref().map(|v| {
            PublicKey::load_level2_pubkey(&header.level2_signed_data.level1_data.level2_key_alg, &v)
        }).transpose()?;

        let end_of_validity = match (
            &header.level2_signed_data.level1_data.end_of_validity_year,
            &header.level2_signed_data.level1_data.end_of_validity_day,
            &header.level2_signed_data.level1_data.end_of_validity_time
        ) {
            (Some(y), Some(d), Some(t)) => {
                let date = match chrono::NaiveDate::from_yo_opt(*y as i32, *d as u32) {
                    Some(t) => t,
                    None => {
                        return Err(BarcodeError::InvalidFormat { msg: "End of Validity out of range".to_string() })
                    }
                };
                let time = match chrono::NaiveTime::from_num_seconds_from_midnight_opt(*t as u32 * 60, 0) {
                    Some(t) => t,
                    None => {
                        return Err(BarcodeError::InvalidFormat { msg: "End of Validity out of range".to_string() })
                    }
                };
                Some(date.and_time(time))
            }
            (None, None, None) => None,
            _ => {
                return Err(BarcodeError::InvalidFormat { msg: "End of Validity out of range".to_string() })
            }
        };

        let validity_duration = match &header.level2_signed_data.level1_data.validity_duration {
            Some(v) => Some(chrono::TimeDelta::seconds(*v as i64)),
            None => None
        };

        Ok(Self {
            version: DosipasVersion::V2,
            level_1_data: rasn::uper::encode(&header.level2_signed_data.level1_data).unwrap(),
            level_1_signature: header.level2_signed_data.level1_signature.unwrap_or_default().to_vec(),
            level_2_signing_alg: header.level2_signed_data.level1_data.level2_signing_alg,
            level_2_public_key,
            security_provider: if let Some(id) = header.level2_signed_data.level1_data.security_provider_num {
                id.to_string()
            } else if let Some(id) = &header.level2_signed_data.level1_data.security_provider_ia5 {
                id.to_string()
            } else {
                return Err(BarcodeError::InvalidFormat { msg: "One of securityProviderNum or securityProviderIA5 must be set".to_string() })
            },
            key_id: header.level2_signed_data.level1_data.key_id.unwrap_or_default(),
            end_of_validity: end_of_validity.map(|t| t.and_utc()),
            validity_duration,
            records: header.level2_signed_data.level1_data.data_sequence.into_iter().map(Into::into).collect::<Vec<_>>(),
            level_2_data: std::sync::RwLock::new(header.level2_signed_data.level2_data.map(Into::into)),
        })
    }
}

impl From<asn1::asn_module_header_v1::DataType> for Record {
    fn from(d: asn1::asn_module_header_v1::DataType) -> Self {
        Self {
            record_id: d.data_format.to_string(),
            data: d.data.to_vec(),
        }
    }
}

impl From<asn1::asn_module_header_v2::DataType> for Record {
    fn from(d: asn1::asn_module_header_v2::DataType) -> Self {
        Self {
            record_id: d.data_format.to_string(),
            data: d.data.to_vec(),
        }
    }
}