#[cfg(test)]
mod patch_data {
    use crate::{IndexedData, SimpleDirectDeltaEncoding};

    #[test]
    fn patch_data() {
        let mut sdd = SimpleDirectDeltaEncoding::new(&[IndexedData::new(0, "Test".as_bytes().to_vec())]);
        let new_data = &[IndexedData::new(0, "Test2".as_bytes().to_vec())];
        let diff_data = sdd.patch(new_data);
        assert_eq!(diff_data, vec![10, 49, 51, 54, 55, 54, 57, 54, 57, 55, 49, 118, 0, 6, 105, 58, 4, 45, 1, 50]);
    }

    #[test]
    fn apply_patch_data() {
        let mut sdd = SimpleDirectDeltaEncoding::new(&[IndexedData::new(0, "Test".as_bytes().to_vec())]);
        let new_data = &[IndexedData::new(0, "Test2".as_bytes().to_vec())];
        let diff_data = sdd.patch(new_data);
        let mut sdd2 = SimpleDirectDeltaEncoding::new(&[IndexedData::new(0, "Test".as_bytes().to_vec())]);
        let data = sdd2.apply_patch(&diff_data).ok();
        let sdd_data = sdd.data_collection.values().map(|x|x.to_owned()).collect::<Vec<IndexedData>>().as_slice().iter().fold(Vec::new(), |mut acc, indexed_data| {
            acc.extend(indexed_data.data.clone());
            acc
        });
        assert_eq!(sdd_data, SimpleDirectDeltaEncoding::fold_indexes(&data.unwrap()));
    }
    
    #[test]
    fn apply_patch_data_1() {
        let mut sdd = SimpleDirectDeltaEncoding::new(&[IndexedData::new(0, "Test".as_bytes().to_vec())]);
        let new_data = &[IndexedData::new(0, "TesNN".as_bytes().to_vec())];
        let diff_data = sdd.patch(new_data);
        let mut sdd2 = SimpleDirectDeltaEncoding::new(&[IndexedData::new(0, "Test".as_bytes().to_vec())]);
        let data = sdd2.apply_patch(&diff_data).ok();
        let sdd_data = sdd.data_collection.values().map(|x|x.to_owned()).collect::<Vec<IndexedData>>().as_slice().iter().fold(Vec::new(), |mut acc, indexed_data| {
            acc.extend(indexed_data.data.clone());
            acc
        });
        assert_eq!(sdd_data, SimpleDirectDeltaEncoding::fold_indexes(&data.unwrap()));
    }

    #[test]
    fn apply_patch_data_from_text_files() {
        apply_diff_from_file("text_1.txt", "text_2.txt");
        apply_diff_from_file("text_1.txt", "text_3.txt");
        apply_diff_from_file("text_2.txt", "text_1.txt");
        apply_diff_from_file("text_2.txt", "text_3.txt");
        apply_diff_from_file("text_3.txt", "text_1.txt");
        apply_diff_from_file("text_3.txt", "text_2.txt");
    }

    fn apply_diff_from_file(file_og: &str, file_diff: &str) -> bool {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let data_path = current_dir.join("test_files");
        let mut og_path = data_path.clone();
        og_path.push(file_og);
        let mut diff_path = data_path.clone();
        diff_path.push(file_diff);

        let original_data_from_file = &[IndexedData::new(0, std::fs::read(og_path).expect("Failed to read original_data.txt"))];
        let diff_data_from_file = &[IndexedData::new(0, std::fs::read(diff_path.clone()).expect("Failed to read new_data.txt"))];

        let mut sdd = SimpleDirectDeltaEncoding::new(diff_data_from_file);
        let diff_data = sdd.patch(original_data_from_file);
        
        let mut sdd2 = SimpleDirectDeltaEncoding::new(diff_data_from_file);
        let data = sdd2.apply_patch(&diff_data).ok();

        let sdd2_data = data.unwrap();

        let sdd_data = sdd.data_collection.values().map(|x|x.to_owned()).collect::<Vec<IndexedData>>().as_slice().iter().fold(Vec::new(), |mut acc, indexed_data| {
            acc.extend(indexed_data.data.clone());
            acc
        });
        assert_eq!(sdd_data.len(), sdd2_data.len());
        assert_eq!(sdd_data, SimpleDirectDeltaEncoding::fold_indexes(&sdd2_data));
        true
    }

    #[test]
    fn patch_all_properties() {
        let props = vec![IndexedData::new(0, "Test".as_bytes().to_vec()), IndexedData::new(1, "Test2".as_bytes().to_vec())];

        let mut sdd = SimpleDirectDeltaEncoding::new(&props);
        let new_data = &[
            IndexedData::new(0, "Test1".as_bytes().to_vec()),
            IndexedData::new(1, "Test3".as_bytes().to_vec())
        ];
        let diff_data = sdd.patch(new_data);

        let mut sdd2 = SimpleDirectDeltaEncoding::new(&props);
        let result_data = sdd2.apply_patch(&diff_data);
        println!("result: {:?}", result_data);
        println!("collection: {:?}", sdd.data_collection);
        assert!(result_data.is_ok());
        let sdd_data = sdd.data_collection.values().map(|x|x.to_owned()).collect::<Vec<IndexedData>>().as_slice().iter().fold(Vec::new(), |mut acc, indexed_data| {
            acc.extend(indexed_data.data.clone());
            acc
        });
        assert_eq!(sdd_data, SimpleDirectDeltaEncoding::fold_indexes(&result_data.unwrap()));
    }

    #[test]
    fn patch_some_properties() {
        let props = vec![IndexedData::new(0, "Test".as_bytes().to_vec()), IndexedData::new(1, "Test2".as_bytes().to_vec())];

        let mut sdd = SimpleDirectDeltaEncoding::new(&props);
        let new_data = &[
            IndexedData::new(1, "Test3".as_bytes().to_vec())
        ];
        let diff_data = sdd.patch(new_data);

        let mut sdd2 = SimpleDirectDeltaEncoding::new(&props);
        let result_data = sdd2.apply_patch(&diff_data);
        assert!(result_data.is_ok());
        let sdd_data = sdd.data_collection.values().map(|x|x.to_owned()).collect::<Vec<IndexedData>>().as_slice().iter().fold(Vec::new(), |mut acc, indexed_data| {
            acc.extend(indexed_data.data.clone());
            acc
        });
        assert_eq!(sdd_data, SimpleDirectDeltaEncoding::fold_indexes(&result_data.unwrap()));
    }

    #[test]
    fn patch_add_property() {
        let props = vec![IndexedData::new(0, "Test".as_bytes().to_vec()), IndexedData::new(1, "Test2".as_bytes().to_vec())];

        let mut sdd = SimpleDirectDeltaEncoding::new(&props);
        let new_data = &[
            IndexedData::new(0, "Test".as_bytes().to_vec()),
            IndexedData::new(1, "Test2".as_bytes().to_vec()),
            IndexedData::new(3, "Test4".as_bytes().to_vec())
        ];
        let diff_data = sdd.patch(new_data);

        let mut sdd2 = SimpleDirectDeltaEncoding::new(&props);
        let result_data = sdd2.apply_patch(&diff_data);
        assert!(result_data.is_ok());
        let sdd_data = sdd.data_collection.values().map(|x|x.to_owned()).collect::<Vec<IndexedData>>().as_slice().iter().fold(Vec::new(), |mut acc, indexed_data| {
            acc.extend(indexed_data.data.clone());
            acc
        });
        assert_eq!(sdd_data, SimpleDirectDeltaEncoding::fold_indexes(&result_data.unwrap()));
    }
    
    #[test]
    fn patch_remove_property() {
        let props = vec![
            IndexedData::new(0, "Test".as_bytes().to_vec()),
            IndexedData::new(1, "Test2".as_bytes().to_vec()),
            IndexedData::new(3, "Test4".as_bytes().to_vec())
        ];

        let mut sdd = SimpleDirectDeltaEncoding::new(&props);
        let new_data = &[
            IndexedData::new(0, "Test".as_bytes().to_vec()),
            IndexedData::new(1, "Test2".as_bytes().to_vec()),
        ];
        let diff_data = sdd.patch(new_data);
        println!("diff_data: {:?}", diff_data);

        let mut sdd2 = SimpleDirectDeltaEncoding::new(&props);
        let result_data = sdd2.apply_patch(&diff_data);
        println!("result: {:?}", result_data);
        println!("collection: {:?}", sdd.data_collection);
        assert!(result_data.is_ok());
        let sdd_data = sdd.data_collection.values().map(|x|x.to_owned()).collect::<Vec<IndexedData>>().as_slice().iter().fold(Vec::new(), |mut acc, indexed_data| {
            acc.extend(indexed_data.data.clone());
            acc
        });
        assert_eq!(sdd_data, SimpleDirectDeltaEncoding::fold_indexes(&result_data.unwrap()));
    }

    #[test]
    fn patch_index_mapping() {
        let props = vec![
            IndexedData::new(0, "Test".as_bytes().to_vec()),
        ];

        let mut sdd = SimpleDirectDeltaEncoding::new(&props);
        sdd.change_index_mapping(0, "t1".as_bytes());
        let new_data = &[
            IndexedData::new(0, "Test2".as_bytes().to_vec()),
        ];
        let diff_data = sdd.patch(new_data);
        println!("diff_data: {:?}", diff_data);

        let mut sdd2 = SimpleDirectDeltaEncoding::new(&props);
        let result_data = sdd2.apply_patch(&diff_data);
        println!("result: {:?}", result_data);
        println!("collection: {:?}", sdd.data_collection);
        assert!(result_data.is_ok());
        let sdd_data = sdd.data_collection.values().map(|x|x.to_owned()).collect::<Vec<IndexedData>>().as_slice().iter().fold(Vec::new(), |mut acc, indexed_data| {
            acc.extend(indexed_data.data.clone());
            acc
        });
        assert_eq!(sdd_data, SimpleDirectDeltaEncoding::fold_indexes(&result_data.unwrap()));
        assert_eq!(sdd.get_index_mapping().get(&0).unwrap().current, sdd2.get_index_mapping().get(&0).unwrap().current);
    }
}