use std::cmp::min;
use chrono::{Datelike, Timelike};

// This value is the same for both GRS80 and WGS84.
const EARTH_CIRCUMFERENCE_METERS: f64 = 40_075_016.7;
const METERS_PER_DEGREE_AT_EQUATOR: f64 = EARTH_CIRCUMFERENCE_METERS / 360.0;

struct DynamicContentDataInner {
    mobile_app_id: Option<rasn::types::Ia5String>,
    timestamp: Option<chrono::DateTime<chrono::Utc>>,
    geo_coordinate: Option<std::sync::Arc<GeoCoordinate>>,
    response_to_challenge: Vec<crate::Record>,
    extension: Option<crate::Record>,
}

pub struct DynamicContentData {
    inner: std::sync::RwLock<DynamicContentDataInner>,
}

struct GeoCoordinateInner {
    coordinate_system: GeoCoordinateSystem,
    position: LatLong,
    accuracy: Option<GeoUnit>
}

pub struct GeoCoordinate {
    inner: std::sync::RwLock<GeoCoordinateInner>,
}

pub struct LatLong {
    pub latitude: f64,
    pub longitude: f64,
}

pub enum GeoCoordinateSystem {
    WGS84,
    GRS80
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum GeoUnit {
    MicroDegree,
    TenthMilliDegree,
    MilliDegree,
    CentiDegree,
    DeciDegree,
}

impl DynamicContentData {
    pub fn new() -> DynamicContentData {
        DynamicContentData {
            inner: std::sync::RwLock::new(DynamicContentDataInner {
                mobile_app_id: None,
                timestamp: None,
                geo_coordinate: None,
                response_to_challenge: vec![],
                extension: None,
            })
        }
    }

    pub fn set_mobile_app_id(&self, app_id: &str) -> Result<(), crate::BarcodeError> {
        self.inner.write().unwrap().mobile_app_id = Some(rasn::types::Ia5String::from_iso646_bytes(app_id.as_bytes())
            .map_err(|e| crate::BarcodeError::InvalidFormat { msg: format!("Invalid mobile app ID: {}", e) })?);
        Ok(())
    }

    pub fn set_timestamp(&self, timestamp: std::time::SystemTime) {
        self.inner.write().unwrap().timestamp = Some(timestamp.into());
    }

    pub fn set_coordinate(&self, coordinate: std::sync::Arc<GeoCoordinate>) {
        self.inner.write().unwrap().geo_coordinate = Some(coordinate);
    }

    pub fn add_response_to_challenge(&self, response: crate::Record) {
        self.inner.write().unwrap().response_to_challenge.push(response);
    }

    pub fn set_extension(&self, extension: crate::Record) {
        self.inner.write().unwrap().extension = Some(extension);
    }

    pub(crate) fn to_asn1(&self) -> Result<crate::asn1::asn_module_dynamic_content_data_v1::UicDynamicContentData, crate::BarcodeError> {
        let inner = self.inner.read().unwrap();
        Ok(crate::asn1::asn_module_dynamic_content_data_v1::UicDynamicContentData {
            dynamic_content_mobile_app_id: inner.mobile_app_id.clone(),
            dynamic_content_time_stamp: inner.timestamp.map(|t| {
                crate::asn1::asn_module_dynamic_content_data_v1::TimeStampData {
                    day: t.ordinal() as u16,
                    time: (t.hour() * 60 + t.minute()) * 60 + t.second(),
                }
            }),
            dynamic_content_geo_coordinate: inner.geo_coordinate.as_ref().map(|v| v.to_asn1()),
            dynamic_content_response_to_challenge: if inner.response_to_challenge.is_empty() {
                None
            } else {
                Some(inner.response_to_challenge.iter().map(|d| Ok(crate::asn1::asn_module_dynamic_content_data_v1::ExtensionData {
                    extension_id: rasn::types::Ia5String::from_iso646_bytes(d.record_id.as_bytes())
                        .map_err(|e| crate::BarcodeError::InvalidFormat { msg: format!("Invalid DCD challenge response ID: {}", e) })?,
                    extension_data: d.data.clone().into(),
                })).collect::<Result<Vec<_>, _>>()?)
            },
            dynamic_content_extension: match &inner.extension {
                Some(d) => Some(crate::asn1::asn_module_dynamic_content_data_v1::ExtensionData {
                    extension_id: rasn::types::Ia5String::from_iso646_bytes(d.record_id.as_bytes())
                        .map_err(|e| crate::BarcodeError::InvalidFormat { msg: format!("Invalid DCD extension ID: {}", e) })?,
                    extension_data: d.data.clone().into(),
                }),
                None => None
            }
        })
    }
}

impl GeoCoordinate {
    pub fn new(position: LatLong) -> Self {
        Self {
            inner: std::sync::RwLock::new(GeoCoordinateInner {
                coordinate_system: GeoCoordinateSystem::WGS84,
                position,
                accuracy: None
            })
        }
    }

    pub fn set_coordinate_system(&self, system: GeoCoordinateSystem) {
        self.inner.write().unwrap().coordinate_system = system;
    }

    pub fn set_accuracy_from_meters(&self, position: LatLong, accuracy_meters: f64) {
        self.inner.write().unwrap().accuracy = Some(GeoUnit::from_accuracy_meters(position, accuracy_meters));
    }

    pub fn set_accuracy_raw(&self, accuracy: GeoUnit) {
        self.inner.write().unwrap().accuracy = Some(accuracy);
    }

    fn degrees_to_precision(value: f64) -> GeoUnit {
        let mirco_degrees = (value * 100_000.0).round() as u64;
        if mirco_degrees % 10 != 0 {
            GeoUnit::MicroDegree
        } else if mirco_degrees % 100 != 0 {
            GeoUnit::TenthMilliDegree
        } else if mirco_degrees % 1_000 != 0 {
            GeoUnit::MilliDegree
        } else if mirco_degrees % 10_000 != 0 {
            GeoUnit::CentiDegree
        } else {
            GeoUnit::DeciDegree
        }
    }

    fn to_asn1(&self) -> crate::asn1::asn_module_dynamic_content_data_v1::GeoCoordinateType {
        let inner = self.inner.read().unwrap();

        let latitude_precision = Self::degrees_to_precision(inner.position.latitude);
        let longitude_precision = Self::degrees_to_precision(inner.position.longitude);
        let precision = min(latitude_precision, longitude_precision);
        let precision_multiplier = match precision {
            GeoUnit::MicroDegree => 100_000.0,
            GeoUnit::TenthMilliDegree => 10_000.0,
            GeoUnit::MilliDegree => 1_000.0,
            GeoUnit::CentiDegree => 100.0,
            GeoUnit::DeciDegree => 10.0,
        };
        let latitude_integer = (inner.position.latitude * precision_multiplier).round() as u64;
        let longitude_integer = (inner.position.longitude * precision_multiplier).round() as u64;

        crate::asn1::asn_module_dynamic_content_data_v1::GeoCoordinateType {
            geo_unit: precision.to_asn1(),
            coordinate_system: match inner.coordinate_system {
                GeoCoordinateSystem::WGS84 => crate::asn1::asn_module_dynamic_content_data_v1::GeoCoordinateSystemType::wgs84,
                GeoCoordinateSystem::GRS80 => crate::asn1::asn_module_dynamic_content_data_v1::GeoCoordinateSystemType::grs80,
            },
            hemisphere_longitude: if inner.position.longitude >= 0.0 {
                crate::asn1::asn_module_dynamic_content_data_v1::HemisphereLongitudeType::east
            } else {
                crate::asn1::asn_module_dynamic_content_data_v1::HemisphereLongitudeType::west
            },
            hemisphere_latitude: if inner.position.latitude >= 0.0 {
                crate::asn1::asn_module_dynamic_content_data_v1::HemisphereLatitudeType::north
            } else {
                crate::asn1::asn_module_dynamic_content_data_v1::HemisphereLatitudeType::south
            },
            longitude: longitude_integer.into(),
            latitude: latitude_integer.into(),
            accuracy: inner.accuracy.map(|v| v.to_asn1()),
        }
    }
}

impl GeoUnit {
    pub fn from_accuracy_meters(position: LatLong, accuracy_meters: f64) -> GeoUnit {
        let meters_per_degree_at_latitude = position.latitude.to_radians().cos() * METERS_PER_DEGREE_AT_EQUATOR;
        let degrees_accuracy = accuracy_meters / meters_per_degree_at_latitude;
        let micro_degrees_accuracy = (degrees_accuracy * 100_000.0).round() as u64;
        if micro_degrees_accuracy <= 1 {
            GeoUnit::MicroDegree
        } else if micro_degrees_accuracy <= 10 {
            GeoUnit::TenthMilliDegree
        } else if micro_degrees_accuracy <= 100 {
            GeoUnit::MilliDegree
        } else if micro_degrees_accuracy <= 1000 {
            GeoUnit::CentiDegree
        } else {
            GeoUnit::DeciDegree
        }
    }

    fn to_asn1(&self) -> crate::asn1::asn_module_dynamic_content_data_v1::GeoUnitType {
        match self {
            Self::MicroDegree => crate::asn1::asn_module_dynamic_content_data_v1::GeoUnitType::microDegree,
            Self::TenthMilliDegree => crate::asn1::asn_module_dynamic_content_data_v1::GeoUnitType::tenthmilliDegree,
            Self::MilliDegree => crate::asn1::asn_module_dynamic_content_data_v1::GeoUnitType::microDegree,
            Self::CentiDegree => crate::asn1::asn_module_dynamic_content_data_v1::GeoUnitType::centiDegree,
            Self::DeciDegree => crate::asn1::asn_module_dynamic_content_data_v1::GeoUnitType::deciDegree,
        }
    }
}