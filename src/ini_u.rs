#[cfg(test)]
mod tests {
	use crate::Ini;



	#[test]
	fn test_from_contents_valid() {
		let contents:&str = "[Category1]\nkey1=value1\nkey2=value2\n\n[Category2]\nkey3=value3\n";
		let ini:Ini = Ini::from_contents(contents);

		println!("{:?}", ini.categories.iter().map(|c| &c.name).collect::<Vec<&String>>());
		assert_eq!(ini.categories.len(), 2);
		assert_eq!(ini.get_category("Category1").unwrap().variables.len(), 2);
		assert_eq!(ini.get_variable("Category1", "key1").unwrap().value, "value1");
		assert_eq!(ini.get_variable("Category2", "key3").unwrap().value, "value3");
	}

	#[test]
	fn test_to_string_encoded_values() {
		let contents:&str = "[Category1]\nkey1=value1\nkey2=value2\n";
		let ini:Ini = Ini::from_contents(contents);
		let encoded:String = ini.to_string();
		
		let expected:&str = "[Category1]\nkey1=value1\nkey2=value2";
		assert_eq!(encoded, expected);
	}

	#[test]
	fn test_save_and_load() {
		let temp_file:&str = "test.ini";
		let contents:&str = "[Category1]\nkey1=value1\nkey2=value2\n";
		let ini:Ini = Ini::from_contents(contents);

		ini.save_to_file(temp_file).unwrap();

		let loaded_ini:Ini = Ini::from_file(temp_file).unwrap();
		assert_eq!(loaded_ini.get_variable("Category1", "key1").unwrap().value, "value1");
		std::fs::remove_file(temp_file).unwrap();
	}

	#[test]
	fn test_empty_file() {
		let contents:&str = "";
		let ini:Ini = Ini::from_contents(contents);

		assert_eq!(ini.categories.len(), 0);
	}

	#[test]
	fn test_invalid_line() {
		let contents:&str = "Invalid line here";
		let ini:Ini = Ini::from_contents(contents);

		assert!(ini.categories.is_empty());
	}

	#[test]
	fn test_malformed_category() {
		let contents:&str = "[Category1\nkey=value\n";
		let ini:Ini = Ini::from_contents(contents);

		assert!(ini.categories[0].name.is_empty());
		assert_eq!(ini.categories[0].variables[0].name, "key");
		assert_eq!(ini.categories[0].variables[0].value, "value");
	}

	#[test]
	fn test_category_without_variables() {
		let contents:&str = "[Category1]\n[Category2]\nkey=value\n";
		let ini:Ini = Ini::from_contents(contents);

		assert!(ini.get_category("Category1").is_none());
		assert_eq!(ini.get_variable("Category2", "key").unwrap().value, "value");
	}

	#[test]
	fn test_special_characters() {
		let contents:&str = "[Special]\nkey=special value!@#$%^&*()";
		let ini:Ini = Ini::from_contents(contents);

		assert_eq!(ini.get_variable("Special", "key").unwrap().value, "special value!@#$%^&*()");
		assert_eq!(ini.to_string(), contents);
	}

	#[test]
	fn test_encoding_decoding() {
		let contents:&str = "[EncodeTest]\nkey1 =hello_world\nkey2 =rust ini\n";
		let ini:Ini = Ini::from_contents_with_encoding(
			contents,
			|name| name.trim().to_string(),
			|name| name.clone(),
			|value| value.replace("_", " "),
			|value| value.replace(" ", "_")
		);

		assert_eq!(ini.get_variable("EncodeTest", "key1").unwrap().value, "hello world");
		assert_eq!(ini.to_string(), "[EncodeTest]\nkey1=hello_world\nkey2=rust_ini");
	}

	#[test]
	fn test_missing_variable() {
		let contents:&str = "[Missing]\n";
		let ini:Ini = Ini::from_contents(contents);

		assert!(ini.get_variable("Missing", "key").is_none());
	}

	#[test]
	fn test_create_variable() {
		let contents:&str = "[Missing]\n";
		let mut ini:Ini = Ini::from_contents(contents);
		ini.set_variable("NotMissing", "key", "value".to_string());

		assert_eq!(ini.get_variable("NotMissing", "key").unwrap().value, "value");
	}

	#[test]
	fn test_generic_type() {
		let contents:&str = "[Category1]\nkey1=1\nkey2=2\n\n[Category2]\nkey3=3";
		let ini:Ini<String, usize> = Ini::<String, usize>::from_contents_with_encoding(
			contents,
			|name| name.to_string(),
			|name| name.clone(),
			|value| value.parse::<usize>().unwrap(),
			usize::to_string
		);

		println!("{:?}", ini.categories.iter().map(|c| &c.name).collect::<Vec<&String>>());
		assert_eq!(ini.categories.len(), 2);
		assert_eq!(ini.get_category("Category1").unwrap().variables.len(), 2);
		assert_eq!(ini.get_variable("Category1", "key1").unwrap().value, 1);
		assert_eq!(ini.get_variable("Category2", "key3").unwrap().value, 3);
	}

	#[test]
	fn test_debug_print() {
		let contents:&str = "[Category1]\nkey1=0\nkey2=1,2\n\n[Category2]\nkey3=3,4\n";
		let ini:Ini<String, Vec<usize>> = Ini::<String, Vec<usize>>::from_contents_with_encoding(
			contents,
			|name| name.to_string(),
			|name| name.clone(),
			|value| value.split(',').map(|number| number.parse::<usize>().unwrap()).collect::<Vec<usize>>(),
			|numbers| numbers.iter().map(|number| number.to_string()).collect::<Vec<String>>().join(", ")
		);

		assert_eq!(&format!("{:?}", ini), "[Category1]\n\"key1\"=[0]\n\"key2\"=[1, 2]\n\n[Category2]\n\"key3\"=[3, 4]");
	}
}