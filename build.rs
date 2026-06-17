use std::path::PathBuf;
use rasn_compiler::OutputMode;
use rasn_compiler::prelude::*;

fn main() {
    uniffi::generate_scaffolding("src/uic-dosipas.udl").unwrap();

    println!("cargo:rerun-if-changed=asn1/uicBarcodeHeader_v1.0.0.asn");
    println!("cargo:rerun-if-changed=asn1/uicBarcodeHeader_v2.0.1.asn");
    println!("cargo:rerun-if-changed=asn1/uicDynamicContentData_v1.0.5.asn");

    Compiler::<RasnBackend, _>::new()
        .add_asn_by_path(PathBuf::from("asn1/uicBarcodeHeader_v1.0.0.asn"))
        .set_output_mode(OutputMode::SingleFile(PathBuf::from("./asn1_gen/barcode_header_v1_0_0.rs")))
        .compile().unwrap();
    Compiler::<RasnBackend, _>::new()
        .add_asn_by_path(PathBuf::from("asn1/uicBarcodeHeader_v2.0.1.asn"))
        .set_output_mode(OutputMode::SingleFile(PathBuf::from("./asn1_gen/barcode_header_v2_0_1.rs")))
        .compile().unwrap();
    Compiler::<RasnBackend, _>::new()
        .add_asn_by_path(PathBuf::from("asn1/uicDynamicContentData_v1.0.5.asn"))
        .set_output_mode(OutputMode::SingleFile(PathBuf::from("./asn1_gen/dynamic_content_data_v1_0_5.rs")))
        .compile().unwrap();
}