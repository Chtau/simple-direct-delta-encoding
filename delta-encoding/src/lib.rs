mod data_difference;
#[cfg(test)]
mod data_difference_tests;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, HashMap};

use data_difference::*;
use dispnet_hash::{DispnetHash, HashType};

#[derive(Debug)]
pub enum SDDEError {
    CRC(String),
    DifferenceInvalid(String),
}

#[derive(Clone)]
pub struct SimpleDirectDeltaEncoding {
    pub data_collection: BTreeMap<u8, IndexedData>,
    pub crc: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct IndexedData {
    pub index: u8,
    pub data: Vec<u8>,
}

impl IndexedData {
    pub fn new(index: u8, data: Vec<u8>) -> IndexedData {
        IndexedData {
            index,
            data,
        }
    }
}

impl SimpleDirectDeltaEncoding {
    pub fn new(data: &[IndexedData]) -> SimpleDirectDeltaEncoding {
        let data = Self::get_sorted(data);
        let bytes = Self::fold_indexed_data(&data);
        let crc = DispnetHash::create(HashType::CRC, &bytes, None);
        let mut data_map: BTreeMap<u8, IndexedData> = BTreeMap::new();
        for indexed_data in data {
            data_map.insert(indexed_data.index, indexed_data.clone());
        }
        SimpleDirectDeltaEncoding {
            data_collection: data_map,
            crc: crc.digest_value.clone(),
        }
    }

    pub fn load(data: &[IndexedData], crc: Vec<u8>) -> SimpleDirectDeltaEncoding {
        let data = Self::get_sorted(data);
        let mut data_map: BTreeMap<u8, IndexedData> = BTreeMap::new();
        for indexed_data in data {
            data_map.insert(indexed_data.index, indexed_data.clone());
        }
        SimpleDirectDeltaEncoding {
            data_collection: data_map, 
            crc 
        }
    }

    /// Patch the data with the new data and return the diff data
    ///
    ///
    /// The diff data is a byte array with the following format:<br/>
    /// [CRC length, CRC value, Difference 1, Difference 2, ...]<br/>
    /// * The CRC length is a single byte that represents the length of the CRC value
    /// * The CRC value is the hash of the data before patching
    /// * The Difference is a struct that represents the difference between the old and new data
    ///
    ///
    /// The Difference is a byte array with the following format:
    /// [Action, Range start, Range length, Value]
    /// The Action is a single byte that represents the action that should be taken
    /// The Range start is a variable length byte array that represents the start index of the range
    /// The Range length is a variable length byte array that represents the length of the range
    /// The Value is a variable length byte array that represents the value that should be inserted
    /// The Value is only present in the Replace and Insert actions
    pub fn patch(&mut self, new_data: &[IndexedData]) -> Vec<u8> {
        let new_data = Self::get_sorted(new_data);
        // add the crc
        let mut diff_data: Vec<u8> = vec![self.crc.len() as u8];
        diff_data.extend(self.crc.clone());

        for data in new_data {
            if let Some(old_data) = self.data_collection.get_mut(&data.index) {
                let last_diff = DataDifference::diff(&old_data.data, &data.data);
                old_data.data = data.data.to_owned();
                // set the index
                diff_data.push(b'v');
                diff_data.push(data.index);
                for diff in last_diff.iter() {
                    let bytes = diff.to_bytes();
                    // add the difference
                    diff_data.extend(Difference::get_usize_type_to_bytes(bytes.len()));
                    diff_data.extend(bytes);
                }
            }
        }

        diff_data
    }

    pub fn apply_patch(&mut self, diff_data: &[u8]) -> Result<Vec<IndexedData>, SDDEError> {
        let crc_length = diff_data[0];
        let crc_value = &diff_data[1..(1 + crc_length as usize)];
        let bytes = Self::fold_indexed_data(self.data_collection.values().map(|x|x.to_owned()).collect::<Vec<IndexedData>>().as_slice());
        let crc = DispnetHash::create(HashType::CRC, &bytes, None);
        if crc.digest_value != crc_value {
            return Err(SDDEError::CRC("CRC value does not match".to_owned()));
        }

        let diffs = Self::get_differences(diff_data);
        for (index, diff) in diffs.iter() {
            if let Some(src_data) = self.data_collection.get_mut(index) {
                let data = DataDifference::apply_diff(&src_data.data, diff);
                src_data.data = data;
            }
        }

        self.crc = crc.digest_value.clone();

        // return the data in order
        let mut data: Vec<IndexedData> = Vec::new();
        for (_index, d) in self.data_collection.iter() {
            data.push(d.clone());
        }
        Ok(data)
    }

    pub fn fold_bytes(bytes: &[Vec<u8>]) -> Vec<u8> {
        bytes.iter().fold(Vec::new(), |mut acc, byte| {
            acc.extend(byte.clone());
            acc
        })
    }

    pub fn fold_indexes(bytes: &[IndexedData]) -> Vec<u8> {
        bytes.iter().fold(Vec::new(), |mut acc, index| {
            acc.extend(index.data.clone());
            acc
        })
    }

    pub fn get_differences(diff_bytes: &[u8]) -> HashMap<u8, Vec<Difference>> {
        let diff_bytes = Self::get_differences_bytes_with_crc(diff_bytes);
        let mut diffs: HashMap<u8, Vec<Difference>> = HashMap::new();
        let mut i = 0;
        let mut index = 0;
        while i < diff_bytes.len() {
            // get index
            if diff_bytes[i] == b'v' {
                index = diff_bytes[i + 1];
                i += 2;    
            }
            
            let diff_length = Difference::get_usize_type_from_bytes(&diff_bytes[i..]);
            i += diff_length.1;
            let bytes = &diff_bytes[i..(i + diff_length.0)];
            let diff = Difference::from_bytes(bytes);
            i += diff_length.0;
            
            if let Some(d) = diffs.get_mut(&index) {
                d.push(diff);
            } else {
                diffs.insert(index, vec![diff]);
            }
        }
        diffs
    }

    pub fn validate_patch_differences(diff_bytes: &[u8]) -> Result<(), SDDEError> {
        let diff_bytes = Self::get_differences_bytes_with_crc(diff_bytes);
        let mut i = 0;
        while i < diff_bytes.len() {
            // get index
            if diff_bytes[i] == b'v' {
                i += 2;    
            }

            let diff_length = Difference::get_usize_type_from_bytes(&diff_bytes[i..]);
            i += diff_length.1;
            let bytes = &diff_bytes[i..(i + diff_length.0)];
            let diff = Difference::validate_from_bytes(bytes);
            if diff.is_err() {
                return Err(diff.err().unwrap());
            }
            i += diff_length.0;
        }
        Ok(())
    }

    pub fn get_differences_bytes_with_crc(diff_bytes: &[u8]) -> &[u8] {
        let crc_length = diff_bytes[0];
        let _crc_value = &diff_bytes[1..(1 + crc_length as usize)];
        &diff_bytes[(1 + crc_length as usize)..]
    }

    fn fold_indexed_data(data: &[IndexedData]) -> Vec<u8> {
        data.iter().fold(Vec::new(), |mut acc, indexed_data| {
            acc.extend(indexed_data.data.clone());
            acc
        })
    }

    fn get_sorted(data: &[IndexedData]) -> Vec<IndexedData> {
        let mut data = data.to_vec();
        data.sort_by(|a, b| a.index.cmp(&b.index));
        data
    }
}
