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

    #[test]
    fn apply_patch_data_from_text_files() {
        apply_diff_from_file("text_og.txt", "text_diff.txt");
        apply_diff_from_file("text_og.txt", "text_diff_2.txt");
    }

    fn apply_diff_from_file(file_og: &str, file_diff: &str) -> bool {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let data_path = current_dir.join("test_files");
        let mut og_path = data_path.clone();
        og_path.push(file_og);
        let mut diff_path = data_path.clone();
        diff_path.push(file_diff);

        let original_data_from_file = std::fs::read(og_path).expect("Failed to read original_data.txt");
        let diff_data_from_file = std::fs::read(diff_path.clone()).expect("Failed to read new_data.txt");

        let mut sdd = SimpleDirectDeltaEncoding::new(diff_data_from_file.clone());
        let diff_data = sdd.patch(&original_data_from_file);
        
        let mut sdd2 = SimpleDirectDeltaEncoding::new(diff_data_from_file.clone());
        let data = sdd2.apply_patch(&diff_data).ok();
        assert_eq!(sdd.data, data.unwrap());
        true
    }
}