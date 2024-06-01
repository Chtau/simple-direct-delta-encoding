mod data_difference;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod data_difference_tests;

use data_difference::*;
use dispnet_hash::{DispnetHash, HashType};

pub enum SDDEError {
    CRC(String)
}

pub struct SimpleDirectDeltaEncoding {
    data: Vec<u8>,
    crc: DispnetHash,
}

impl SimpleDirectDeltaEncoding {
    pub fn new(data: Vec<u8>) -> SimpleDirectDeltaEncoding {
        let crc = DispnetHash::create(HashType::CRC, &data, None);
        SimpleDirectDeltaEncoding {
            data,
            crc,
        }
    }

    /// Patch the data with the new data and return the diff data
    /// 
    /// 
    /// The diff data is a byte array with the following format:
    /// [CRC length, CRC value, Difference 1, Difference 2, ...]
    /// The CRC length is a single byte that represents the length of the CRC value
    /// The CRC value is the hash of the data before patching
    /// The Difference is a struct that represents the difference between the old and new data
    /// 
    /// 
    /// The Difference is a byte array with the following format:
    /// [Action, Range start, Range length, Value]
    /// The Action is a single byte that represents the action that should be taken
    /// The Range start is a variable length byte array that represents the start index of the range
    /// The Range length is a variable length byte array that represents the length of the range
    /// The Value is a variable length byte array that represents the value that should be inserted
    /// The Value is only present in the Replace and Insert actions
    pub fn patch(&mut self, new_data: &[u8]) -> Vec<u8> {
        let last_diff = DataDifference::diff(&self.data, new_data);
        self.data = new_data.to_vec();
        
        let mut diff_data: Vec<u8> = vec![self.crc.digest_length as u8];
        diff_data.extend(self.crc.digest_value.clone());
        for diff in last_diff.iter() {
            let bytes = diff.to_bytes();
            diff_data.extend(Difference::get_usize_type_to_bytes(bytes.len()));
            diff_data.extend(bytes);
        }
        diff_data
    }

    pub fn apply_patch(&mut self, diff_data: &[u8]) -> Result<Vec<u8>, SDDEError> {
        let crc_length = diff_data[0];
        let crc_value = &diff_data[1..(1 + crc_length as usize)];
        let diff_bytes = &diff_data[(1 + crc_length as usize)..];
        let crc = DispnetHash::create(HashType::CRC, &self.data, None);
        if crc.digest_value != crc_value {
            return Err(SDDEError::CRC("CRC value does not match".to_owned()));
        }

        let mut diffs: Vec<Difference> = Vec::new();
        let mut i = 0;
        while i < diff_bytes.len() {
            let diff_length = Difference::get_usize_type_from_bytes(&diff_bytes[i..]);
            i += diff_length.1;
            let bytes = &diff_bytes[i..(i + diff_length.0)];
            let diff = Difference::from_bytes(bytes);
            i += diff_length.0;
            diffs.push(diff);
        }
        
        let data = DataDifference::apply_diff(&self.data, &diffs);
        self.data = data;
        self.crc = crc;

        Ok(self.data.clone())
    }
}