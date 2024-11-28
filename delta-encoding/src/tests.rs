#[cfg(test)]
mod patch_data {
    use std::collections::BTreeMap;

    use crate::{IndexedData, SimpleDirectDeltaEncoding};

    #[test]
    fn patch_data() {
        let mut sdd =
            SimpleDirectDeltaEncoding::new(&[IndexedData::new(0, "Test".as_bytes().to_vec())]);
        let new_data = &[IndexedData::new(0, "Test2".as_bytes().to_vec())];
        let diff_data = sdd.patch(new_data);
        assert_eq!(
            diff_data,
            vec![10, 49, 51, 54, 55, 54, 57, 54, 57, 55, 49, 118, 0, 6, 105, 58, 4, 45, 1, 50]
        );
    }

    #[test]
    fn apply_patch_data() {
        let mut sdd =
            SimpleDirectDeltaEncoding::new(&[IndexedData::new(0, "Test".as_bytes().to_vec())]);
        let new_data = &[IndexedData::new(0, "Test2".as_bytes().to_vec())];
        let diff_data = sdd.patch(new_data);
        let mut sdd2 =
            SimpleDirectDeltaEncoding::new(&[IndexedData::new(0, "Test".as_bytes().to_vec())]);
        _ = sdd2.apply_patch(&diff_data).ok();
        assert_eq!(
            fold_data_collection(&sdd.data_collection),
            fold_data_collection(&sdd2.data_collection)
        );
    }

    #[test]
    fn apply_patch_data_1() {
        let mut sdd =
            SimpleDirectDeltaEncoding::new(&[IndexedData::new(0, "Test".as_bytes().to_vec())]);
        let new_data = &[IndexedData::new(0, "TesNN".as_bytes().to_vec())];
        let diff_data = sdd.patch(new_data);
        let mut sdd2 =
            SimpleDirectDeltaEncoding::new(&[IndexedData::new(0, "Test".as_bytes().to_vec())]);
        _ = sdd2.apply_patch(&diff_data).ok();
        assert_eq!(
            fold_data_collection(&sdd.data_collection),
            fold_data_collection(&sdd2.data_collection)
        );
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

        let original_data_from_file = &[IndexedData::new(
            0,
            std::fs::read(og_path).expect("Failed to read original_data.txt"),
        )];
        let diff_data_from_file = &[IndexedData::new(
            0,
            std::fs::read(diff_path.clone()).expect("Failed to read new_data.txt"),
        )];

        let mut sdd = SimpleDirectDeltaEncoding::new(diff_data_from_file);
        let diff_data = sdd.patch(original_data_from_file);

        let mut sdd2 = SimpleDirectDeltaEncoding::new(diff_data_from_file);
        _ = sdd2.apply_patch(&diff_data).ok();

        assert_eq!(
            fold_data_collection(&sdd.data_collection),
            fold_data_collection(&sdd2.data_collection)
        );
        true
    }

    #[test]
    fn patch_all_properties() {
        let props = vec![
            IndexedData::new(0, "Test".as_bytes().to_vec()),
            IndexedData::new(1, "Test2".as_bytes().to_vec()),
        ];

        let mut sdd = SimpleDirectDeltaEncoding::new(&props);
        let new_data = &[
            IndexedData::new(0, "Test1".as_bytes().to_vec()),
            IndexedData::new(1, "Test3".as_bytes().to_vec()),
        ];
        let diff_data = sdd.patch(new_data);

        let mut sdd2 = SimpleDirectDeltaEncoding::new(&props);
        let result_data = sdd2.apply_patch(&diff_data);
        println!("result: {:?}", result_data);
        println!("collection: {:?}", sdd.data_collection);
        assert!(result_data.is_ok());
        assert_eq!(
            fold_data_collection(&sdd.data_collection),
            fold_data_collection(&sdd2.data_collection),
        );
    }

    #[test]
    fn patch_some_properties() {
        let props = vec![
            IndexedData::new(0, "Test".as_bytes().to_vec()),
            IndexedData::new(1, "Test2".as_bytes().to_vec()),
        ];

        let mut sdd = SimpleDirectDeltaEncoding::new(&props);
        let new_data = &[IndexedData::new(1, "Test3".as_bytes().to_vec())];
        let diff_data = sdd.patch(new_data);

        let mut sdd2 = SimpleDirectDeltaEncoding::new(&props);
        let result_data = sdd2.apply_patch(&diff_data);
        assert!(result_data.is_ok());
        assert_eq!(
            fold_data_collection(&sdd.data_collection),
            fold_data_collection(&sdd2.data_collection),
        );
    }

    #[test]
    fn patch_add_property() {
        let props = vec![
            IndexedData::new(0, "Test".as_bytes().to_vec()),
            IndexedData::new(1, "Test2".as_bytes().to_vec()),
        ];

        let mut sdd = SimpleDirectDeltaEncoding::new(&props);
        let new_data = &[
            IndexedData::new(0, "Test".as_bytes().to_vec()),
            IndexedData::new(1, "Test2".as_bytes().to_vec()),
            IndexedData::new(3, "Test4".as_bytes().to_vec()),
        ];
        let diff_data = sdd.patch(new_data);

        let mut sdd2 = SimpleDirectDeltaEncoding::new(&props);
        let result_data = sdd2.apply_patch(&diff_data);
        assert!(result_data.is_ok());
        assert_eq!(
            fold_data_collection(&sdd.data_collection),
            fold_data_collection(&sdd2.data_collection),
        );
    }

    #[test]
    fn patch_remove_property() {
        let props = vec![
            IndexedData::new(0, "Test".as_bytes().to_vec()),
            IndexedData::new(1, "Test2".as_bytes().to_vec()),
            IndexedData::new(3, "Test4".as_bytes().to_vec()),
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
        assert_eq!(
            fold_data_collection(&sdd.data_collection),
            fold_data_collection(&sdd2.data_collection),
        );
    }

    #[test]
    fn patch_index_mapping() {
        let props = vec![IndexedData::new(0, "Test".as_bytes().to_vec())];

        let mut sdd = SimpleDirectDeltaEncoding::new(&props);
        sdd.change_index_mapping(0, "t1".as_bytes());
        let new_data = &[IndexedData::new(0, "Test2".as_bytes().to_vec())];
        let diff_data = sdd.patch(new_data);
        println!("diff_data: {:?}", diff_data);

        let mut sdd2 = SimpleDirectDeltaEncoding::new(&props);
        let result_data = sdd2.apply_patch(&diff_data);
        println!("result: {:?}", result_data);
        println!("collection: {:?}", sdd.data_collection);
        assert!(result_data.is_ok());
        assert_eq!(
            fold_data_collection(&sdd.data_collection),
            fold_data_collection(&sdd2.data_collection),
        );
        assert_eq!(
            sdd.get_index_mapping().get(&0).unwrap().current,
            sdd2.get_index_mapping().get(&0).unwrap().current
        );
    }

    fn fold_data_collection(data_collection: &BTreeMap<u8, IndexedData>) -> Vec<u8> {
        SimpleDirectDeltaEncoding::fold_index(&data_collection.values().cloned().collect::<Vec<_>>())
    }

    #[test]
    fn json_object_patch() {
        let json_source = r#"{"name": "John", "age": 30, "city": ""}"#;
        let json_changes = r#"{"firstname": "John", "age": 30, "city": "New York"}"#;

        let json_source = serde_json::from_str::<serde_json::Value>(json_source).unwrap();
        let json_changes = serde_json::from_str::<serde_json::Value>(json_changes).unwrap();

        let mut src_data = vec![];
        let mut changes_data = vec![];

        for (index, (key, value)) in json_source.as_object().unwrap().iter().enumerate() {
            src_data.push((key, IndexedData::new(
                index as u8,
                value.to_string().as_bytes().to_vec(),
            )));
        }

        for (index, (key, value)) in json_changes.as_object().unwrap().iter().enumerate() {
            changes_data.push((key, IndexedData::new(
                index as u8,
                value.to_string().as_bytes().to_vec(),
            )));
        }

        // setup the source data with already applied index mappings
        let mut sdd = SimpleDirectDeltaEncoding::new(&src_data.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>());
        for (key, value) in src_data.iter() {
            sdd.change_index_mapping(value.index, key.as_bytes());
        }
        sdd.apply_index_mappings();

        // set the changes data beginning with the index mappings and then create a patch
        for (key, value) in changes_data.iter() {
            sdd.change_index_mapping(value.index, key.as_bytes());
        }
        let patch_data = sdd.patch(&changes_data.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>());

        // apply the patch to a new instance of the SDD which only has the source data
        let mut sdd2 = SimpleDirectDeltaEncoding::new(&src_data.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>());
        for (key, value) in src_data.iter() {
            sdd2.change_index_mapping(value.index, key.as_bytes());
        }
        sdd2.apply_index_mappings();
        // apply patch
        let result_data = sdd2.apply_patch(&patch_data);

        // check if the result is ok
        assert!(result_data.is_ok());
        // check if the data is the same
        assert_eq!(
            fold_data_collection(&sdd.data_collection),
            fold_data_collection(&sdd2.data_collection),
        );

        // create a new json object from the data in sdd2
        let mut json_obj = serde_json::Map::new();
        let index_mapping = sdd2.get_index_mapping();
        for (key, value) in sdd2.data_collection.iter() {
            let key = std::str::from_utf8(&index_mapping.get(key).unwrap().current).unwrap();
            if let Ok(num) = std::str::from_utf8(&value.data).unwrap().parse::<i64>() {
                json_obj.insert(key.to_string(), serde_json::Value::Number(serde_json::Number::from(num)));
            } else {
                let a = std::str::from_utf8(&value.data).unwrap().to_string();
                json_obj.insert(key.to_string(), serde_json::Value::String(a.trim_matches('"').to_string()));
            }
        }

        // expected json object should be the same as the changes data
        // json_obj: {"age": Number(30), "city": String("New York"), "firstname": String("John")}
        println!("json_obj: {:?}", json_obj);
        assert_eq!(&json_obj, json_changes.as_object().unwrap());

    }
}
