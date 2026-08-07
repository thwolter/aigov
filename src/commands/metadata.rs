use crate::cli;
use crate::ooxml::OoxmlPackage;
use crate::ooxml::properties::read_metadata;

pub fn show_metadata(filepath: &str, metadata_args: &cli::MetadataArgs) {
    println!("Metadata {}:", filepath);
    let package = OoxmlPackage::open(filepath).unwrap();
    let metadata = read_metadata(&package);
    println!("{:#?}", metadata);
}