# Simple direct delta encoding

Implementation of a custom [delta encoding](https://en.wikipedia.org/wiki/Delta_encoding) with support of key value mapping. The design idea is to have delta encoding which is simple to use with a blob of data but also with a collection of key value pairs.

The library takes data as collection of `IndexedData` which contains an index and bytes. This allows to split the data into as small or large chunks as needed. Every `IndexedData` can be mapped to a named key which can be changed at anytime and the changes will also be provided in the patch. This allows to have some sort of key value pair like structur (JSON object, etc.) to be synced with changes to the keys and values.

## Features

* Patch with differences between indexed data
* CRC checksum to validate patch target
* Apply patch to indexed data
* Index mapping to named keys
* Patches can remove named keys and indexed data
* Patches can add named keys and indexed data

## Usage

### Create and apply patch

```rust
// create patch bytes
let mut sdd = SimpleDirectDeltaEncoding::new(&[IndexedData::new(0, "Test".as_bytes().to_vec())]);
let new_data = &[IndexedData::new(0, "Test2".as_bytes().to_vec())];
let patch_bytes = sdd.patch(new_data);

// apply patch bytes
let mut sdd2 = SimpleDirectDeltaEncoding::new(&[IndexedData::new(0, "Test".as_bytes().to_vec())]);
let changed_data = sdd2.apply_patch(patch_bytes);
// sdd2 will have now the value "Test2" at index 0
```

### Map data to index

Index mapping can be used to have a named key for the index. This is useful when you have changing key value pairs and want to create a patch which also contains changes to the name of the key.

```rust
let props = vec![IndexedData::new(0, "Test".as_bytes().to_vec())];

let mut sdd = SimpleDirectDeltaEncoding::new(&props);
// create index mapping
sdd.change_index_mapping(0, "t1".as_bytes());
let new_data = &[IndexedData::new(0, "Test2".as_bytes().to_vec())];
let patch_data = sdd.patch(new_data);

// apply patch bytes
let mut sdd2 = SimpleDirectDeltaEncoding::new(&props);
let changed_data = sdd2.apply_patch(&patch_data);
// sdd2 will have now a mapping for the index 0 to the named key "t1" with the value "Test2" 
```

### Json object patch

Here is a more complex example where a JSON object is used to create a patch and apply it to a new instance of the SDD.
The changes JSON contains a key name change from `name` to `firstname` and a value change for the key `city`.

```rust
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
// apply index mappings is used here to prevent the patch from including previous index mappings (this is because we set the initial index mappings here)
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
// apply index mappings is used here to prevent the patch from including previous index mappings (this is because we set the initial index mappings here)
sdd2.apply_index_mappings();
// apply patch
let result_data = sdd2.apply_patch(&patch_data);

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
```

## Build instructions for delta encoding

To build, use the following command:

```bash
cargo build -p delta-encoding
```

To run the tests, use the following command:

```bash
cargo test -p delta-encoding
```

To run the benchmarks, use the following command:

```bash
cargo bench -p delta-encoding
```

## Build instructions for web page

To build, use the following command:

```bash
cargo build -p web-page
```

To run the web-page, use the following command:

```bash
cd web-page
trunk serve --open
```
