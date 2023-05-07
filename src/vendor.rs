use std::{
    process,
    collections::HashMap,
    fs::File,
    io::{BufWriter, BufReader, Error},
};

use pnet_datalink::MacAddr;
use csv::Reader;
use speedy::{Readable, Writable};

// The Vendor structure performs search operations on a vendor database to find
// which MAC address belongs to a specific vendor. All network vendors have a
// dedicated MAC address range that is registered by the IEEE and maintained in
// the OUI database. An OUI is a 24-bit globally unique assigned number
// referenced by various standards.

// This is default value if the location of the ieee-oui
// file is not provided in the command line
// This file has already been deserialized and serialized
// back into bincode format for faster loading
pub static IEEE_OUI_PATH: &'static str     = "./data/";
pub static IEEE_OUI_FILE_BIN: &'static str = "ieee-oui.data";
pub static IEEE_OUI_FILE_CSV: &'static str = "ieee-oui.csv";

// Use hashmap for fast recovery of MAC and corresponding
// company that is assigned that MAC
// Key: MAC
// Value: Company informaton
#[derive(Readable, Writable)]
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


    pub fn new(path: Option<&str>) -> Self {
        // maybe only accept .data (bincode) files here?
        // and load and parse .csv in the update function?
        let file                             = get_file(path, IEEE_OUI_FILE_BIN, false);
        let file_buffer                      = BufReader::new(file);
        let vendor                           = Readable::read_from_stream_buffered(file_buffer).unwrap();
        vendor
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

// helper function to reduce redundant code
fn get_file(path: Option<&str>, file_type: &'static str, file_create: bool) -> File {
    // maybe only accept .data (bincode) files here?
    // and load and parse .csv in the update function?
    let file_path = path.unwrap_or(IEEE_OUI_PATH);
    let file: Result<File, Error>;

    match file_create {
        true      => file = File::create(file_path.to_owned() + file_type),
        false     => file = File::open(file_path.to_owned() + file_type),
    }

    let file = match file {
        Ok(file) => file,
        Err(e)   => {
            eprintln!("Error opening ieee-oui file: {:#?}", e);
            std::process::exit(1)
        },
    };

    file
}

// get ieee-oui file from web and convert to bincode .data file
pub fn update(path: Option<&str>) {
    // println!("------- UPDATING IEEE-OUI FILE FROM WEB -------");
    
    let file = get_file(path, IEEE_OUI_FILE_CSV, false);
    let mut rdr = Reader::from_reader(file);
    // parse .csv file and insert records into hashmap
    let mut records = HashMap::new();
    for result in rdr.deserialize::<RawRecord>() {
        // We must tell Serde what type we want to deserialize into.
        match result {
            Ok(record) => records.insert(record.1, record.2 + &record.3),
            Err(e)     => {
                eprintln!("Error deserializing ieee-oui: {:#?}", e);
                std::process::exit(1)
            },
        };
    }
    // take the hashmap and serialize using bincode to a binary file
    // for easy loading later
    let vendor             = Vendor { records };
    // let file               = get_file(path, IEEE_OUI_FILE_BIN, true);
    // let mut buf_write      = BufWriter::new(file);
    vendor.write_to_file("./data/ieee-oui.data");

    // match serial_result {
    //     Ok(_)  => println!("------- UPDATING COMPLETE -------"),
    //     Err(e) => eprintln!("Error updating ieee-oui from web: {:#?}", e)
    // }
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
