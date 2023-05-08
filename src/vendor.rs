use std::{
    collections::HashMap,
    fs,
    fs::File,
    io::{BufWriter, BufReader},
};

use anyhow::Result;
use bincode::{serialize_into, deserialize_from};
use pnet_datalink::MacAddr;
use csv::Reader;
use serde::{Serialize, Deserialize};
use log::{info, error};

// The Vendor structure performs search operations on a vendor database to find
// which MAC address belongs to a specific vendor. All network vendors have a
// dedicated MAC address range that is registered by the IEEE and maintained in
// the OUI database. An OUI is a 24-bit globally unique assigned number
// referenced by various standards.

// This is default value if the location of the ieee-oui
// file is not provided in the command line
// This file has already been deserialized and serialized
// back into bincode format for faster loading
pub static IEEE_OUI_WEB: &'static str      = "http://standards-oui.ieee.org/oui/oui.csv";
pub static IEEE_OUI_PATH: &'static str     = "./data/";
pub static IEEE_OUI_FILE_BIN: &'static str = "./data/ieee-oui.data";
pub static IEEE_OUI_FILE_CSV: &'static str = "./data/ieee-oui.csv";

// Use hashmap for fast recovery of MAC and corresponding
// company that is assigned that MAC
// Key: MAC
// Value: Company informaton
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Vendor {
    pub records: HashMap<String, String>,
}

// Deserialize the ieee-oui.csv to this format
// and then pick and choose which to insert into
// hashmap with values:
// RawRecord.0 = MAL
// RawRecord.1 = MAC
// RawRecord.2 = Company Name
// RawRecord.3 = Company Address
type RawRecord = (String, String, String, String);
impl Vendor {


    pub fn new() -> Self {
        // check if data directory exists
        // create directory and download ieee-oui file otherwise
        if let Err(_) = fs::read_dir(IEEE_OUI_PATH) {
            info!("Data directory doesn't exist. Will create and update.");
            update();
        }
        
        info!("CWD: {:?}", std::env::current_dir());
        
        let file                             = File::open(IEEE_OUI_FILE_BIN).expect("Error opening data file");
        let file_buffer                      = BufReader::new(file);
        let records: HashMap<String, String> = deserialize_from(file_buffer).expect("This shouldn't error unless the .data file was modified manually.");

        Vendor { records }
    }

    pub fn has_vendor_db(&self) -> bool {
        self.records.len() > 0
    }

    pub fn search_by_mac(&mut self, mac_address: &MacAddr) -> Option<String> {
        let oui = format!("{:02X}{:02X}{:02X}", mac_address.0, mac_address.1, mac_address.2);
        // dbg!(&oui);
        match self.records.get(&oui) {
            Some(company) => Some(company.to_string()),
            None          => None,
        }
    }
}

pub async fn update() -> Result<()>{
    info!("------- UPDATING IEEE-OUI FILE FROM WEB -------");

    // Check if data directory exists,
    // create the directory if it doesn't
    if let Err(_) = fs::read_dir(IEEE_OUI_PATH) {
        fs::create_dir(IEEE_OUI_PATH)?;
    }
    
    // Download ieee-oui file to new directory
    let response = reqwest::get(IEEE_OUI_WEB).await?;
    let content = response.text().await?;
    
    // parse .csv file and insert records into hashmap
    let mut rdr = Reader::from_reader(content.as_bytes());
    let mut records = HashMap::new();
    for result in rdr.deserialize::<RawRecord>() {
        // We must tell Serde what type we want to deserialize into.
        match result {
            Ok(record) => records.insert(record.1, record.2 + &record.3),
            Err(e)     => {
                error!("Error deserializing ieee-oui: {:#?}", e);
                std::process::exit(1)
            },
        };
    }
    // take the hashmap and serialize using bincode to a binary file
    // for easy loading later
    let vendor             = Vendor { records };
    let file               = File::create(IEEE_OUI_FILE_BIN)?;
    let mut buf_write      = BufWriter::new(file);
    serialize_into(&mut buf_write, &vendor)?;

    info!("------- UPDATING COMPLETE -------");
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_create_vendor_resolver() {
        
        let vendor = Vendor::new("./data/ieee-oui.csv");

        assert_eq!(vendor.has_vendor_db(), true);
    }

    #[test]
    fn should_handle_unresolved_database() {
        
        let vendor = Vendor::new("./unknown.csv");

        assert_eq!(vendor.has_vendor_db(), false);
    }

    #[test]
    fn should_find_specific_mac_vendor() {
        
        let mut vendor = Vendor::new("./data/ieee-oui.csv");
        let mac = MacAddr::new(0x40, 0x55, 0x82, 0xc3, 0xe5, 0x5b);

        assert_eq!(vendor.search_by_mac(&mac), Some("Nokia".to_string()));
    }

    #[test]
    fn should_find_first_mac_vendor() {
        
        let mut vendor = Vendor::new("./data/ieee-oui.csv");
        let mac = MacAddr::new(0x00, 0x22, 0x72, 0xd7, 0xb5, 0x23);

        assert_eq!(vendor.search_by_mac(&mac), Some("American Micro-Fuel Device Corp.".to_string()));
    }

    #[test]
    fn should_find_last_mac_vendor() {
        
        let mut vendor = Vendor::new("./data/ieee-oui.csv");
        let mac = MacAddr::new(0xcc, 0x9d, 0xa2, 0x14, 0x2e, 0x6f);

        assert_eq!(vendor.search_by_mac(&mac), Some("Eltex Enterprise Ltd.".to_string()));
    }

    #[test]
    fn should_handle_unknown_mac_vendor() {
        
        let mut vendor = Vendor::new("./data/ieee-oui.csv");
        let mac = MacAddr::new(0xbb, 0xbb, 0xbb, 0xd2, 0xf5, 0xb6);

        assert_eq!(vendor.search_by_mac(&mac), None);
    }

    #[test]
    fn should_pad_correctly_with_zeroes() {
        
        let mut vendor = Vendor::new("./data/ieee-oui.csv");
        let mac = MacAddr::new(0x01, 0x01, 0x01, 0x67, 0xb2, 0x1d);

        assert_eq!(vendor.search_by_mac(&mac), Some("SomeCorp".to_string()));
    }
}
