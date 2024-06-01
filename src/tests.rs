#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn diff_replace_action() {
        let data_old = "Test1".as_bytes();
        let data_new = "Test2".as_bytes();
        let diff = SimpleDirectDeltaEncoding::diff(data_old, data_new);
        println!("{:?} String: {:?}", diff, std::str::from_utf8(&diff.last().unwrap().value).unwrap());
        assert_eq!(diff.last().unwrap().to_bytes(), vec![b'r', b':', 4, b'-', 1, b'2']);
    }
    
    #[test]
    fn diff_insert_action() {
        let data_old = "Test".as_bytes();
        let data_new = "Test2".as_bytes();
        let diff = SimpleDirectDeltaEncoding::diff(data_old, data_new);
        println!("{:?} String: {:?}", diff, std::str::from_utf8(&diff.last().unwrap().value).unwrap());
        assert_eq!(diff.last().unwrap().to_bytes(), vec![b'i', b':', 4, b'-', 1, b'2']);
    }
        
    #[test]
    fn diff_delete_action() {
        let data_old = "Test2".as_bytes();
        let data_new = "Test".as_bytes();
        let diff = SimpleDirectDeltaEncoding::diff(data_old, data_new);
        println!("{:?} String: {:?}", diff, std::str::from_utf8(&diff.last().unwrap().value).unwrap());
        assert_eq!(diff.last().unwrap().to_bytes(), vec![b'd', b':', 4, b'-', 1]);
    }
    
    #[test]
    fn diff_replace_action_with_same_inbetween() {
        let data_old = "Test1THELLO".as_bytes();
        let data_new = "Test2Thello".as_bytes();
        let diff = SimpleDirectDeltaEncoding::diff(data_old, data_new);
        println!("{:?} String: {:?}", diff, std::str::from_utf8(&diff.last().unwrap().value).unwrap());
        assert_eq!(diff.last().unwrap().to_bytes(), vec![b'r', b':', 4, b'-', 7, b'2', b'T', b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn diff_large_data_sets() {
        let data_old = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let data_new = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 11];
        let diff = SimpleDirectDeltaEncoding::diff(&data_old, &data_new);
        println!("{:?} String: {:?}", diff, std::str::from_utf8(&diff.last().unwrap().value).unwrap());
        assert_eq!(diff.last().unwrap().to_bytes(), vec![b'r', b':', 9, b'-', 1, 11]);
    }

    #[test]
    fn diff_multiple_actions() {
        let data_old = vec![1, 2, 3, 4, 5];
        let data_new = vec![1, 2, 6, 7, 8, 9];
        let diff = SimpleDirectDeltaEncoding::diff(&data_old, &data_new);
        println!("{:?} String: {:?}", diff, std::str::from_utf8(&diff.last().unwrap().value).unwrap());
        assert_eq!(diff[0].to_bytes(), vec![b'r', b':', 2, b'-', 3, 6, 7, 8]);
        assert_eq!(diff[1].to_bytes(), vec![b'i', b':', 5, b'-', 1, 9]);
    }

    #[test]
    fn diff_no_changes() {
        let data_old = vec![1, 2, 3, 4, 5];
        let data_new = vec![1, 2, 3, 4, 5];
        let diff = SimpleDirectDeltaEncoding::diff(&data_old, &data_new);
        assert_eq!(diff.len(), 0);
    }

    #[test]
    fn diff_range_length_u16() {
        let data_old = vec![0; 300];
        let data_new = vec![3; 300];
        let diff = SimpleDirectDeltaEncoding::diff(&data_old, &data_new);
        assert_eq!(diff.last().unwrap().to_bytes()[..20], [vec![b'r', b':', 0, b'-', b's', 1, 44], vec![3; 300]].concat()[..20]);
    }

    #[test]
    fn diff_range_length_u32() {
        let data_old = vec![0; 65_536];
        let data_new = vec![3; 65_536];
        let diff = SimpleDirectDeltaEncoding::diff(&data_old, &data_new);
        assert_eq!(diff.last().unwrap().to_bytes()[..20], [vec![b'r', b':', 0, b'-', b'i', 0, 1, 0, 0], vec![3; 65_536]].concat()[..20]);
    }

    #[test]
    #[ignore]
    fn diff_range_length_u64() {
        let data_old = vec![0; 4_294_967_296];
        let data_new = vec![3; 4_294_967_296];
        let diff = SimpleDirectDeltaEncoding::diff(&data_old, &data_new);
        assert_eq!(diff.last().unwrap().to_bytes()[..20], [vec![b'r', b':', 0, b'-', b'l', 0, 0, 0, 1, 0, 0, 0, 0], vec![3; 4_294_967_296]].concat()[..20]);
    }

    #[test]
    fn from_bytes_replace_action() {
        let bytes = vec![b'r', b':', 4, b'-', 1, b'2'];
        let difference = Difference::from_bytes(&bytes);
        assert_eq!(difference.action, DifferenceAction::Replace);
        assert_eq!(difference.range, Range::new(4, 1));
        assert_eq!(difference.value, vec![b'2']);
        assert!(!difference.is_open);
    }

    #[test]
    fn from_bytes_insert_action() {
        let bytes = vec![b'i', b':', 4, b'-', 1, b'2'];
        let difference = Difference::from_bytes(&bytes);
        assert_eq!(difference.action, DifferenceAction::Insert);
        assert_eq!(difference.range, Range::new(4, 1));
        assert_eq!(difference.value, vec![b'2']);
    }

    #[test]
    fn from_bytes_delete_action() {
        let bytes = vec![b'd', b':', 4, b'-', 1];
        let difference = Difference::from_bytes(&bytes);
        assert_eq!(difference.action, DifferenceAction::Delete);
        assert_eq!(difference.range, Range::new(4, 1));
        assert_eq!(difference.value, vec![]);
    }

    #[test]
    fn from_bytes_replace_action_with_same_inbetween() {
        let bytes = vec![b'r', b':', 4, b'-', 7, b'2', b'T', b'h', b'e', b'l', b'l', b'o'];
        let difference = Difference::from_bytes(&bytes);
        assert_eq!(difference.action, DifferenceAction::Replace);
        assert_eq!(difference.range, Range::new(4, 7));
        assert_eq!(difference.value, vec![b'2', b'T', b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn from_bytes_large_data_sets() {
        let bytes = vec![b'r', b':', 9, b'-', 1, 11];
        let difference = Difference::from_bytes(&bytes);
        assert_eq!(difference.action, DifferenceAction::Replace);
        assert_eq!(difference.range, Range::new(9, 1));
        assert_eq!(difference.value, vec![11]);
    }

    #[test]
    fn from_bytes_multiple_actions() {
        let bytes1 = vec![b'r', b':', 2, b'-', 3, 6, 7, 8];
        let difference1 = Difference::from_bytes(&bytes1);
        assert_eq!(difference1.action, DifferenceAction::Replace);
        assert_eq!(difference1.range, Range::new(2, 3));
        assert_eq!(difference1.value, vec![6, 7, 8]);

        let bytes2 = vec![b'i', b':', 5, b'-', 1, 9];
        let difference2 = Difference::from_bytes(&bytes2);
        assert_eq!(difference2.action, DifferenceAction::Insert);
        assert_eq!(difference2.range, Range::new(5, 1));
        assert_eq!(difference2.value, vec![9]);
    }

    #[test]
    fn from_bytes_range_length_u16() {
        let bytes = [vec![b'r', b':', 0, b'-', b's', 1, 44], vec![3; 300]].concat();
        let difference = Difference::from_bytes(&bytes);
        assert_eq!(difference.action, DifferenceAction::Replace);
        assert_eq!(difference.range, Range::new(0, 300));
        assert_eq!(difference.value, vec![3; 300]);
    }

    #[test]
    fn from_bytes_range_length_u32() {
        let bytes = [vec![b'r', b':', 0, b'-', b'i', 0, 1, 0, 0], vec![3; 65_536]].concat();
        let difference = Difference::from_bytes(&bytes);
        assert_eq!(difference.action, DifferenceAction::Replace);
        assert_eq!(difference.range, Range::new(0, 65_536));
        assert_eq!(difference.value, vec![3; 65_536]);
    }

    #[test]
    #[ignore]
    fn from_bytes_range_length_u64() {
        let bytes = [vec![b'r', b':', 0, b'-', b'l', 0, 0, 0, 1, 0, 0, 0, 0], vec![3; 4_294_967_296]].concat();
        let difference = Difference::from_bytes(&bytes);
        assert_eq!(difference.action, DifferenceAction::Replace);
        assert_eq!(difference.range, Range::new(0, 4_294_967_296));
        assert_eq!(difference.value, vec![3; 4_294_967_296]);
    }

    #[test]
    fn apply_diff_replace_action() {
        let data_old = "Test1".as_bytes();
        let data_new = "Test2".as_bytes();
        let diff = SimpleDirectDeltaEncoding::diff(data_old, data_new);
        let mut data = data_old.to_owned();
        SimpleDirectDeltaEncoding::apply_diff(&mut data, &diff);
        assert_eq!(data, data_new);
    }
}