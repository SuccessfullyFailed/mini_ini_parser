#[cfg(test)]
mod tests {
	use std::error::Error;
	use crate::IniCore;



	/* HELPER METHODS */

	fn simple_encoder(value:&str) -> String {
		value.replace(" ", "_")
	}

	fn simple_decoder(value:&str) -> String {
		value.replace("_", " ")
	}



	/* TEST METHODS */

	#[test]
	fn test_from_contents_valid() {
		let contents:&str = "[Category1]\nkey1=value1\nkey2=value2\n\n[Category2]\nkey3=value3\n";
		let ini:IniCore = IniCore::from_contents(contents, &simple_encoder, &simple_decoder).unwrap();

		println!("{:?}", ini.categories().iter().map(|c| &c.name).collect::<Vec<&String>>());
		assert_eq!(ini.categories().len(), 2);
		assert_eq!(ini["Category1"].data.len(), 2);
		assert_eq!(ini["Category1"]["key1"].value, "value1");
		assert_eq!(ini["Category2"]["key3"].value, "value3");
	}

	#[test]
	fn test_to_string_encoded_values() {
		let contents:&str = "[Category1]\nkey1=value1\nkey2=value2\n";
		let ini:IniCore = IniCore::from_contents(contents, &simple_encoder, &simple_decoder).unwrap();
		let encoded:String = ini.to_string_encoded_values();
		
		let expected:&str = "[Category1]\nkey1=value1\nkey2=value2";
		assert_eq!(encoded, expected);
	}

	#[test]
	fn test_save_and_load() {
		let temp_file:&str = "test.ini";
		let contents:&str = "[Category1]\nkey1=value1\nkey2=value2\n";
		let ini:IniCore = IniCore::from_contents(contents, &simple_encoder, &simple_decoder).unwrap();

		ini.save_to_file(temp_file).unwrap();

		let loaded_ini:IniCore = IniCore::from_file(temp_file, &simple_encoder, &simple_decoder).unwrap();
		assert_eq!(loaded_ini["Category1"]["key1"].value, "value1");
		std::fs::remove_file(temp_file).unwrap();
	}

	#[test]
	fn test_empty_file() {
		let contents:&str = "";
		let ini:IniCore = IniCore::from_contents(contents, &simple_encoder, &simple_decoder).unwrap();

		assert_eq!(ini.categories().len(), 0);
	}

	#[test]
	fn test_invalid_line() {
		let contents:&str = "Invalid line here";
		let ini:Result<IniCore, Box<dyn Error>> = IniCore::from_contents(contents, &simple_encoder, &simple_decoder);

		assert!(ini.is_err());
	}

	#[test]
	fn test_malformed_category() {
		let contents:&str = "[Category1\nkey=value\n";
		let ini:Result<IniCore, Box<dyn Error>> = IniCore::from_contents(contents, &simple_encoder, &simple_decoder);

		assert!(ini.is_err());
	}

	#[test]
	fn test_category_without_variables() {
		let contents:&str = "[Category1]\n[Category2]\nkey=value\n";
		let ini:IniCore = IniCore::from_contents(contents, &simple_encoder, &simple_decoder).unwrap();

		assert_eq!(ini["Category1"].data.len(), 0);
		assert_eq!(ini["Category2"]["key"].value, "value");
	}

	#[test]
	fn test_special_characters() {
		let contents:&str = "[Special]\nkey=special_value!@#$%^&*()";
		let ini:IniCore = IniCore::from_contents(contents, &simple_encoder, &simple_decoder).unwrap();

		assert_eq!(ini["Special"]["key"].value, "special value!@#$%^&*()");
	}

	#[test]
	fn test_encoding_decoding() {
		let contents:&str = "[EncodeTest]\nkey1=hello world\nkey2=rust ini\n";
		let ini:IniCore = IniCore::from_contents(contents, &simple_encoder, &simple_decoder).unwrap();

		assert_eq!(ini["EncodeTest"]["key1"].value, "hello world");
		assert_eq!(ini.to_string_encoded_values(), "[EncodeTest]\nkey1=hello_world\nkey2=rust_ini");
	}

	#[test]
	fn test_missing_variable() {
		let contents:&str = "[Missing]\n";
		let ini:IniCore = IniCore::from_contents(contents, &simple_encoder, &simple_decoder).unwrap();

		assert!(!ini["Missing"]["key"].is_ok());
	}
}