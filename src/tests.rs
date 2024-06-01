#[cfg(test)]
mod test {
    use crate::SimpleDirectDeltaEncoding;

    #[test]
    fn patch_data() {
        let mut sdd = SimpleDirectDeltaEncoding::new("Test".as_bytes().to_vec());
        let new_data = "Test2".as_bytes();
        let diff_data = sdd.patch(new_data);
        assert_eq!(diff_data, vec![10, 49, 51, 54, 55, 54, 57, 54, 57, 55, 49, 6, 105, 58, 4, 45, 1, 50]);
    }

    #[test]
    fn apply_patch_data() {
        let mut sdd = SimpleDirectDeltaEncoding::new("Test".as_bytes().to_vec());
        let new_data = "Test2".as_bytes();
        let diff_data = sdd.patch(new_data);
        let mut sdd2 = SimpleDirectDeltaEncoding::new("Test".as_bytes().to_vec());
        let data = sdd2.apply_patch(&diff_data).ok();
        assert_eq!(sdd.data, data.unwrap());
    }
}