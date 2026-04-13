use std::error::Error;
use file_ref::FileRef;



const DEFAULT_DECODER:fn(&str) -> String = |v| v.to_string();
const DEFAULT_ENCODER:fn(&String) -> String = |v| v.to_string();
pub trait ValueDecoder<ValueType>:Fn(&str) -> ValueType + Send + Sync + 'static {}
pub trait ValueEncoder<ValueType>:Fn(&ValueType) -> String + Send + Sync + 'static {}
impl<ValueType, T:Fn(&str) -> ValueType + Send + Sync + 'static> ValueDecoder<ValueType> for T {}
impl<ValueType, T:Fn(&ValueType) -> String + Send + Sync + 'static> ValueEncoder<ValueType> for T {}



pub struct Ini<ValueType:'static> {
	pub categories:Vec<IniCategory<ValueType>>,
	source_file:Option<FileRef>,
	_value_decoder:Box<dyn ValueDecoder<ValueType>>,
	value_encoder:Box<dyn ValueEncoder<ValueType>>
}
impl<ValueType> Ini<ValueType> {

	/* CONSTRUCTOR METHODS */

	/// Create a new ini from a file.
	/// Reads the file and parses the file immediately.
	pub fn from_file(file_path:&str) -> Result<Ini<String>, Box<dyn Error>> {
		Ini::from_file_with_encoding(file_path, DEFAULT_DECODER, DEFAULT_ENCODER)
	}

	/// Create a new ini from a file and encoding settings.
	/// Reads the file and parses the file immediately.
	pub fn from_file_with_encoding<Decoder:ValueDecoder<ValueType>, Encoder:ValueEncoder<ValueType>>(file_path:&str, value_decoder:Decoder, value_encoder:Encoder) -> Result<Self, Box<dyn Error>> {
		let file:FileRef = FileRef::new(file_path);
		Ok(
			Ini {
				categories: Self::parse_raw_contents(&file.read()?, &value_decoder),
				source_file: Some(file),
				_value_decoder: Box::new(value_decoder),
				value_encoder: Box::new(value_encoder)
			}
		)
	}

	/// Create a new ini from raw contents.
	/// Parses the contents immediately.
	pub fn from_contents(contents:&str) -> Ini<String> {
		Ini::from_contents_with_encoding(contents, DEFAULT_DECODER, DEFAULT_ENCODER)
	}

	/// Create a new ini from raw contents and encoding settings..
	/// Parses the contents immediately.
	pub fn from_contents_with_encoding<Decoder:ValueDecoder<ValueType>, Encoder:ValueEncoder<ValueType>>(contents:&str, decoder:Decoder, encoder:Encoder) -> Self {
		Ini {
			categories: Self::parse_raw_contents(&contents, &decoder),
			source_file: None,
			_value_decoder: Box::new(decoder),
			value_encoder: Box::new(encoder)
		}
	}

	/// Save the ini to a specific file path.
	pub fn save_to_file(&self, file_path:&str) -> Result<(), Box<dyn Error>> {
		FileRef::new(file_path).write(self.to_string())
	}

	/// Save the ini to the file path it originally came from.
	/// Will return an error if the ini did not come from a file.
	pub fn save_changes(&self) -> Result<(), Box<dyn Error>> {
		match &self.source_file {
			Some(file) => file.write(self.to_string()),
			None => Err("Could not safe ini to origin file, ini did not come from a file.".into())
		}
	}



	/* GETTER AND SETTER METHODS */

	/// Try to find a category in the ini.
	pub fn get_category(&self, category_name:&str) -> Option<&IniCategory<ValueType>> {
		self.categories.iter().find(|category| category.name == category_name)
	}

	/// Try to find a mutable category in the ini.
	pub fn get_category_mut(&mut self, category_name:&str) -> Option<&mut IniCategory<ValueType>> {
		self.categories.iter_mut().find(|category| category.name == category_name)
	}

	/// Try to find a variable in the ini.
	pub fn get_variable(&self, category_name:&str, variable_name:&str) -> Option<&IniVariable<ValueType>> {
		self.get_category(category_name).and_then(|category| category.get_variable(variable_name))
	}

	/// Try to find a mutable variable in the ini.
	pub fn get_variable_mut(&mut self, category_name:&str, variable_name:&str) -> Option<&mut IniVariable<ValueType>> {
		self.get_category_mut(category_name).and_then(|category| category.get_variable_mut(variable_name))
	}



	/* PARSING METHODS */

	/// Parse raw contents into ini categories.
	fn parse_raw_contents(contents:&str, value_decoder:&dyn Fn(&str) -> ValueType) -> Vec<IniCategory<ValueType>> {
		const CATEGORY_OPEN:char = '[';
		const CATEGORY_CLOSE:char = ']';
		const VAR_SPLITTER:char = '=';
		
		let mut categories:Vec<IniCategory<ValueType>> = vec![IniCategory::new("")];
		for line in contents.split(['\n', '\r']) {
			let trimmed_line:&str = line.trim();
			if trimmed_line.is_empty() {
				continue;
			}
			if trimmed_line.starts_with(CATEGORY_OPEN) && trimmed_line.ends_with(CATEGORY_CLOSE) {
				let category_name:&str = &trimmed_line[1..trimmed_line.len() - 1];
				categories.push(IniCategory::new(category_name));
			}
			if let Some(split_index) = line.chars().position(|char| char == VAR_SPLITTER) {
				let var_name:&str = &line[..split_index];
				let var_value:&str = &line[split_index + 1..];
				categories.last_mut().unwrap().variables.push(IniVariable::new(var_name, value_decoder(var_value)));
			}
		}
		categories.retain(|category| !category.variables.is_empty());
		categories
	}
}
impl<ValueType> ToString for Ini<ValueType> {
	fn to_string(&self) -> String {
		self.categories.iter().map(|category| category.to_encoded_string(&self.value_encoder)).collect::<Vec<String>>().join("\n\n")
	}
}



pub struct IniCategory<ValueType> {
	pub name:String,
	pub variables:Vec<IniVariable<ValueType>>
}
impl<ValueType> IniCategory<ValueType> {

	/* CONSTRUCTOR METHODS */

	/// Create a new category.
	pub fn new(name:&str) -> IniCategory<ValueType> {
		IniCategory {
			name: name.to_string(),
			variables: Vec::new()
		}
	}



	/* GETTER AND SETTER METHODS */

	/// Try to find a variable in the category.
	pub fn get_variable(&self, variable_name:&str) -> Option<&IniVariable<ValueType>> {
		self.variables.iter().find(|variable| variable.name == variable_name)
	}

	/// Try to find a mutable variable in the category.
	pub fn get_variable_mut(&mut self, variable_name:&str) -> Option<&mut IniVariable<ValueType>> {
		self.variables.iter_mut().find(|variable| variable.name == variable_name)
	}



	/* PARSING METHODS */

	/// Turn self into a string with encoded values.
	pub fn to_encoded_string(&self, value_encoder:&dyn ValueEncoder<ValueType>) -> String {
		format!("[{}]\n{}", self.name, self.variables.iter().map(|variable| variable.to_encoded_string(value_encoder)).collect::<Vec<String>>().join("\n"))
	}
}



pub struct IniVariable<ValueType> {
	pub name:String,
	pub value:ValueType
}
impl<ValueType> IniVariable<ValueType> {

	/* CONSTRUCTOR METHODS */

	/// Create a new variable.
	pub fn new(name:&str, value:ValueType) -> Self {
		IniVariable {
			name: name.to_string(),
			value
		}
	}



	/* PARSING METHODS */

	/// Turn self into a string with encoded values.
	pub fn to_encoded_string(&self, encoder:&dyn ValueEncoder<ValueType>) -> String {
		format!("{}={}", self.name, encoder(&self.value))
	}
}