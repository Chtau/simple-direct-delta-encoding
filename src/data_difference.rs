#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DifferenceAction {
    Replace,
    Insert,
    Delete,
}

impl From<u8> for DifferenceAction {
    fn from(value: u8) -> Self {
        match value {
            b'r' => DifferenceAction::Replace,
            b'i' => DifferenceAction::Insert,
            b'd' => DifferenceAction::Delete,
            _ => panic!("Invalid difference action")
        }
    }
}

impl From<DifferenceAction> for u8 {
    fn from(val: DifferenceAction) -> Self {
        match val {
            DifferenceAction::Replace => b'r',
            DifferenceAction::Insert => b'i',
            DifferenceAction::Delete => b'd',
        }
    }
}

/// range indicator start-length (in byte conversion a prefix is used for the usize type [implicit if no other short key matches it is default u8 value])
#[derive(Debug, PartialEq)]
pub struct Range {
    pub start: usize,
    pub length: usize,
}

impl Range {
    pub fn new(start: usize, length: usize) -> Self {
        Self {
            start,
            length,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum USizeType {
    /// '' = byte/u8 [0-255]
    U8,
    /// s = short/u16 [0-65,535]
    U16,
    /// i = int/u32 [0-4,294,967,295]
    U32,
    /// l = long/u64 [0-18,446,744,073,709,551,615]
    U64,
}

impl From<u8> for USizeType {
    fn from(value: u8) -> Self {
        match value {
            b's' => USizeType::U16,
            b'i' => USizeType::U32,
            b'l' => USizeType::U64,
            _ => USizeType::U8,
        }
    }
}

impl From<USizeType> for u8 {
    fn from(val: USizeType) -> Self {
        match val {
            USizeType::U16 => b's',
            USizeType::U32 => b'i',
            USizeType::U64 => b'l',
            _ => b'u',
        }
    }
}

#[derive(Debug)]
pub struct Difference {
    pub action: DifferenceAction,
    pub range: Range,
    pub value: Vec<u8>,
    pub is_open: bool,
}

impl Difference {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut diff = Vec::new();
        diff.push(self.action.into());
        diff.push(b':');
        diff.extend(Self::get_usize_type_to_bytes(self.range.start));
        diff.push(b'-');
        diff.extend(Self::get_usize_type_to_bytes(self.range.length));
        if self.action != DifferenceAction::Delete {
            diff.extend(self.value.to_owned());
        }
        diff
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let action: DifferenceAction = bytes[0].into();
        let range_start = Self::get_usize_type_from_bytes(&bytes[2..]);
        let offset = 2 + range_start.1 + 1; // 2 bytes for action and range start, range start bytes count, 1 byte for range separator
        let range_end = Self::get_usize_type_from_bytes(&bytes[offset..]);
        let offset = offset + range_end.1;
        let value = bytes[offset..].to_vec();
        let is_open = false;
        Self {
            action,
            range: Range::new(range_start.0, range_end.0),
            value,
            is_open,
        }
    }

    pub fn get_usize_type_to_bytes(value: usize) -> Vec<u8> {
        if value <= 255 {
            (value as u8).to_be_bytes().to_vec()
        } else if value <= 65_535 {
            [vec![USizeType::U16.into()], (value as u16).to_be_bytes().to_vec()].concat()
        } else if value <= 4_294_967_295 {
            [vec![USizeType::U32.into()], (value as u32).to_be_bytes().to_vec()].concat()
        } else {
            [vec![USizeType::U64.into()], (value as u64).to_be_bytes().to_vec()].concat()
        }
    }

    /// Returns the usize value and the bytes count of the usize type
    pub fn get_usize_type_from_bytes(bytes: &[u8]) -> (usize, usize) {
        match bytes[0].into() {
            USizeType::U16 => {
                let mut buffer = [0; 2];
                buffer.copy_from_slice(&bytes[1..3]);
                (u16::from_be_bytes(buffer) as usize, 3)
            },
            USizeType::U32 => {
                let mut buffer = [0; 4];
                buffer.copy_from_slice(&bytes[1..5]);
                (u32::from_be_bytes(buffer) as usize, 5)
            },
            USizeType::U64 => {
                let mut buffer = [0; 8];
                buffer.copy_from_slice(&bytes[1..9]);
                (u64::from_be_bytes(buffer) as usize, 9)
            },
            _ => (bytes[0] as usize, 1)
        }
    }
}

pub struct DataDifference { }

impl DataDifference {
    pub fn diff(old_data: &[u8], new_data: &[u8]) -> Vec<Difference> {
        let mut differences: Vec<Difference> = Vec::new();
        let mut same_count = 0;
        let mut same_byte_buffer: Vec<u8> = Vec::new();
        
        for (i, b) in new_data.iter().enumerate() {
            let old_b = old_data.get(i);
            if let Some(old_byte) = old_b {
                // replace action
                if old_byte != b {

                    if let Some(diff) = differences.last_mut() {
                        if !diff.is_open {
                            differences.push(Difference {
                                action: DifferenceAction::Replace,
                                range: Range::new(i, 1),
                                value: vec![*b],
                                is_open: true,
                            });
                        } else {
                            // if we have a same byte buffer, we append it to the value
                            if same_count > 0 {
                                diff.value.extend(same_byte_buffer.clone());
                                diff.range.length += same_count;
                                same_byte_buffer.clear();
                                same_count = 0;
                            }
                            diff.value.push(*b);
                            diff.range.length += 1;
                        }
                    } else {
                        differences.push(Difference {
                            action: DifferenceAction::Replace,
                            range: Range::new(i, 1),
                            value: vec![*b],
                            is_open: true,
                        });
                    }
                } else {
                    // old and current are the same
                    // if we have a diff start index and we have differences

                    if let Some(diff) = differences.last_mut() {
                        if same_count > 1 {
                            // if we have more than 1 same byte, we close the difference structure
                            diff.is_open = false;

                            same_count = 0;
                            same_byte_buffer.clear();
                        } else {
                            same_count += 1;
                            same_byte_buffer.push(*b);
                        }
                    }
                }
            } else {
                // insert action
                if let Some(diff) = differences.last_mut() {
                    if !diff.is_open {
                        differences.push(Difference {
                            action: DifferenceAction::Insert,
                            range: Range::new(i, 1),
                            value: vec![*b],
                            is_open: true,
                        });
                    } else {
                        // if open and is insert, we append the value
                        if diff.action == DifferenceAction::Insert {
                            diff.value.push(*b);
                            diff.range.length += 1;
                        } else {
                            // if open and is not insert, we close the difference and create a new insert
                            same_count = 0;
                            same_byte_buffer.clear();
                            diff.is_open = false;
                            differences.push(Difference {
                                action: DifferenceAction::Insert,
                                range: Range::new(i, 1),
                                value: vec![*b],
                                is_open: true,
                            });
                        }
                    }
                } else {
                    differences.push(Difference {
                        action: DifferenceAction::Insert,
                        range: Range::new(i, 1),
                        value: vec![*b],
                        is_open: true,
                    });
                }
            }
        }

        // close remaining open differences
        if let Some(diff) = differences.last_mut() {
            diff.is_open = false;
        }

        // check for deletion on the end
        if old_data.len() > new_data.len() {
            differences.push(Difference {
                action: DifferenceAction::Delete,
                range: Range::new(new_data.len(), old_data.len() - new_data.len()),
                value: vec![],
                is_open: false,
            });
        }

        differences
    }

    pub fn apply_diff(data: &[u8], diff: &[Difference]) -> Vec<u8> {
        let mut data = data.to_vec();
        for d in diff {
            if d.action == DifferenceAction::Replace {
                data[d.range.start..(d.range.start + d.range.length)].copy_from_slice(&d.value);       
            } else if d.action == DifferenceAction::Insert {
                if data.len() > d.range.start {
                    data[d.range.start..(d.range.start + 1)].copy_from_slice(&d.value);
                } else {
                    data.extend_from_slice(&d.value);
                }
            } else if d.action == DifferenceAction::Delete {
                data.drain(d.range.start..(d.range.start + d.range.length));
            }
        }
        data
    }
}