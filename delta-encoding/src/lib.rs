mod data_difference;
#[cfg(test)]
mod data_difference_tests;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

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
    index_mapping: BTreeMap<u8, Vec<u8>>,
    last_index_mapping: BTreeMap<u8, HistoryValue>,
}

#[derive(Debug, Clone, Default)]
pub struct HistoryValue {
    pub current: Vec<u8>,
    pub last: Vec<u8>,
}

impl HistoryValue {
    pub fn new(current: Vec<u8>) -> HistoryValue {
        HistoryValue {
            current,
            last: Vec::new(),
        }
    }

    pub fn set(&mut self, current: Vec<u8>) {
        self.last = self.current.clone();
        self.current = current;
    }
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

#[derive(Debug, Default)]
pub struct EntryDifference {
    pub remove_entry: bool,
    pub diffs: Vec<Difference>,
    pub map_name_changed: Option<Vec<Difference>>,
}

impl EntryDifference {
    pub fn new(diffs: Vec<Difference>) -> EntryDifference {
        EntryDifference {
            remove_entry: false,
            diffs,
            map_name_changed: None,
        }
    }

    pub fn remove_entry() -> EntryDifference {
        EntryDifference {
            remove_entry: true,
            diffs: Vec::new(),
            map_name_changed: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexedDataResult {
    pub index: u8,
    pub data: Vec<u8>,
    pub map_name_changed: Option<Vec<u8>>,
}

impl IndexedDataResult {
    pub fn new(index_data: &IndexedData) -> IndexedDataResult {
        IndexedDataResult {
            index: index_data.index,
            data: index_data.data.clone(),
            map_name_changed: None,
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
            index_mapping: BTreeMap::new(),
            last_index_mapping: BTreeMap::new(),
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
            crc,
            index_mapping: BTreeMap::new(),
            last_index_mapping: BTreeMap::new(),
        }
    }

    pub fn change_index_mapping(&mut self, index: u8, key: &[u8]) {
        self.index_mapping.insert(index, key.to_owned());
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
        let new_indexes: Vec<u8> = new_data.iter().map(|x| x.index).collect();
        // add the crc
        let mut diff_data: Vec<u8> = vec![self.crc.len() as u8];
        diff_data.extend(self.crc.clone());

        for data in new_data {
            if let Some(old_data) = self.data_collection.get_mut(&data.index) {
                let last_diff = DataDifference::diff(&old_data.data, &data.data);
                old_data.data = data.data.to_owned();
                
                // create the diff data for the index
                let mut new_diff = Vec::new();
                // set the index
                new_diff.push(b'v');
                new_diff.push(data.index);
                for diff in last_diff.iter() {
                    let bytes = diff.to_bytes();
                    // add the difference
                    new_diff.extend(Difference::get_usize_type_to_bytes(bytes.len()));
                    new_diff.extend(bytes);
                }
                // only add the diff if there are any changes to the data (first 2 bytes are the index)
                if new_diff.len() > 2 {
                    diff_data.extend(new_diff);
                }
            } else {
                // add the new data entry
                self.data_collection.insert(data.index, data.clone());
                // create the diff data for the index
                let mut new_diff = Vec::new();
                // set the index
                new_diff.push(b'v');
                new_diff.push(data.index);
                for diff in DataDifference::diff(&Vec::new(), &data.data).iter() {
                    let bytes = diff.to_bytes();
                    // add the difference
                    new_diff.extend(Difference::get_usize_type_to_bytes(bytes.len()));
                    new_diff.extend(bytes);
                }
                diff_data.extend(new_diff);
            }
        }

        // check if there are indexes removed
        let src_indexes: Vec<u8> = self.data_collection.keys().copied().collect();
        let removed_indexes: Vec<u8> = src_indexes.iter().filter(|x| !new_indexes.contains(x)).copied().collect();

        for index in &removed_indexes {
            // remove the index from the data collection
            self.data_collection.remove(index);

            // add the remove index command to the patch           
            diff_data.extend(vec![
                b'v',
                index.to_owned(),
                b'r',
            ]);
        }

        // apply changes in the index mapping to the last index mapping
        for (index, new_data) in self.index_mapping.iter() {
            // ignore mappings for removed indexes
            if removed_indexes.contains(index) {
                continue;
            }
            // create the diff data for the index
            // set the index and the control byte
            let new_diff = vec![
                b'v',
                *index,
                b'm',
            ];

            let mut is_new_mapping = false;
            // add the diff data for the index mapping to the patch
            if let Some(old_value) = self.last_index_mapping.get(index) {
                let last_diff = DataDifference::diff(&old_value.current, new_data);

                let mut diff_only = Vec::new();
                for diff in last_diff.iter() {
                    let bytes = diff.to_bytes();
                    // add the difference
                    diff_only.extend(Difference::get_usize_type_to_bytes(bytes.len()));
                    diff_only.extend(bytes);
                }
                // only add the diff if there are any changes to the data (first 2 bytes are the index)
                if !diff_only.is_empty() {
                    diff_data.extend(new_diff);
                    diff_data.extend(Difference::get_usize_type_to_bytes(diff_only.len()));
                    diff_data.extend(diff_only);
                }
            } else {
                is_new_mapping = true;
                // add the new data entry
                let mut diff_only = Vec::new();
                for diff in DataDifference::diff(&Vec::new(), new_data).iter() {
                    let bytes = diff.to_bytes();
                    // add the difference
                    diff_only.extend(Difference::get_usize_type_to_bytes(bytes.len()));
                    diff_only.extend(bytes);
                }
                diff_data.extend(new_diff);
                diff_data.extend(Difference::get_usize_type_to_bytes(diff_only.len()));
                diff_data.extend(diff_only);
            }
            // update the last index mapping
            if is_new_mapping {
                self.last_index_mapping.insert(*index, HistoryValue::new(new_data.clone()));
            } else {
                self.last_index_mapping.get_mut(index).unwrap().set(new_data.clone());
            }
        }
        self.index_mapping.clear();

        diff_data
    }

    pub fn apply_patch(&mut self, diff_data: &[u8]) -> Result<Vec<IndexedDataResult>, SDDEError> {
        let crc_length = diff_data[0];
        let crc_value = &diff_data[1..(1 + crc_length as usize)];
        let bytes = Self::fold_indexed_data(self.data_collection.values().map(|x|x.to_owned()).collect::<Vec<IndexedData>>().as_slice());
        let crc = DispnetHash::create(HashType::CRC, &bytes, None);
        if crc.digest_value != crc_value {
            return Err(SDDEError::CRC("CRC value does not match".to_owned()));
        }

        let mut return_data: Vec<IndexedDataResult> = Vec::new();
        let diffs = Self::get_differences(diff_data);
        for (index, diff) in diffs.iter() {
            // the entry should be removed
            if diff.remove_entry {
                self.data_collection.remove(index);
                continue;
            }
            if let Some(src_data) = self.data_collection.get_mut(index) {
                let data = DataDifference::apply_diff(&src_data.data, &diff.diffs);
                src_data.data = data;
            } else {
                // the index does not exist, add a new data entry
                let new_data = Vec::new();
                let data = DataDifference::apply_diff(&new_data, &diff.diffs);
                let data = IndexedData::new(*index, data);
                self.data_collection.insert(*index, data);
            }


            let mut index_data = IndexedDataResult::new(self.data_collection.get(index).unwrap());
            // check if the map name has changes
            if diff.map_name_changed.is_some() {
                let last_index_map_bytes = self.last_index_mapping.get(index).cloned().unwrap_or_default().current;
                let map_diffs_bytes = DataDifference::apply_diff(&last_index_map_bytes, diff.map_name_changed.as_ref().unwrap());
                index_data.map_name_changed = Some(map_diffs_bytes.clone());
                // update the last index mapping
                if !self.last_index_mapping.contains_key(index) {
                    self.last_index_mapping.insert(*index, HistoryValue::new(map_diffs_bytes));
                } else {
                    self.last_index_mapping.get_mut(index).unwrap().set(map_diffs_bytes);
                }
            }
            return_data.push(index_data);
        }

        self.crc = crc.digest_value.clone();

        Ok(return_data)
    }

    pub fn fold_bytes(bytes: &[Vec<u8>]) -> Vec<u8> {
        bytes.iter().fold(Vec::new(), |mut acc, byte| {
            acc.extend(byte.clone());
            acc
        })
    }

    pub fn fold_indexes(bytes: &[IndexedDataResult]) -> Vec<u8> {
        bytes.iter().fold(Vec::new(), |mut acc, index| {
            acc.extend(index.data.clone());
            acc
        })
    }

    pub fn get_differences(diff_bytes: &[u8]) -> BTreeMap<u8, EntryDifference> {
        Self::on_get_differences(diff_bytes, true)
    }

    pub fn get_index_mapping(&self) -> BTreeMap<u8, HistoryValue> {
        self.last_index_mapping.clone()
    }

    fn on_get_differences(diff_bytes: &[u8], has_crc: bool) -> BTreeMap<u8, EntryDifference> {
        let diff_bytes = if has_crc { Self::get_differences_bytes_with_crc(diff_bytes) } else { diff_bytes };
        let mut diffs: BTreeMap<u8, EntryDifference> = BTreeMap::new();
        let mut i = 0;
        let mut index = 0;
        while i < diff_bytes.len() {
            // get index
            if diff_bytes[i] == b'v' {
                index = diff_bytes[i + 1];
                i += 2;    
            }
            // handle remove entry
            if diff_bytes[i] == b'r' {
                i += 1;
                diffs.insert(index, EntryDifference::remove_entry());
                continue;
            }
            // handle map name changes
            if diff_bytes[i] == b'm' {
                i += 1;
                let map_diffs_length = Difference::get_usize_type_from_bytes(&diff_bytes[i..]);
                i += map_diffs_length.1;
                let bytes = &diff_bytes[i..(i + map_diffs_length.0)];
                i += bytes.len();

                // get the diffs for the map name
                let map_diffs_result = Self::on_get_differences(bytes, false);
                let map_entry_diff = map_diffs_result.get(&0);
                if map_entry_diff.is_none() {
                    continue;
                }
                let map_entry_diff = map_entry_diff.unwrap().to_owned().diffs.clone();

                // insert the map name changed diff
                if let Some(d) = diffs.get_mut(&index) {
                    d.map_name_changed = Some(map_entry_diff);
                } else {
                    diffs.insert(index, EntryDifference::new(Vec::new()));
                    diffs.get_mut(&index).unwrap().map_name_changed = Some(map_entry_diff);
                }

                continue;
            }
            
            let diff_length = Difference::get_usize_type_from_bytes(&diff_bytes[i..]);
            i += diff_length.1;
            let bytes = &diff_bytes[i..(i + diff_length.0)];
            let diff = Difference::from_bytes(bytes);
            i += diff_length.0;
            
            if let Some(d) = diffs.get_mut(&index) {
                d.diffs.push(diff);
            } else {
                diffs.insert(index, EntryDifference::new(vec![diff]));
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
