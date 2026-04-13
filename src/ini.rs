use std::{ error::Error, ops::{ Index, IndexMut } };
use file_ref::FileRef;



const DEFAULT_DECODER:fn(&str) -> String = |v| v.to_string();
const DEFAULT_ENCODER:fn(&str) -> String = |v| v.to_string();
pub trait ValueDecoder:Fn(&str) -> String + Send + Sync + 'static {}
pub trait ValueEncoder:Fn(&str) -> String + Send + Sync + 'static {}
impl<T:Fn(&str) -> String + Send + Sync + 'static> ValueDecoder for T {}
impl<T:Fn(&str) -> String + Send + Sync + 'static> ValueEncoder for T {}



pub struct Ini {
	pub categories:Vec<IniCategory>,
	source_file:Option<FileRef>,
	value_encoder:Box<dyn ValueEncoder>,
	_value_decoder:Box<dyn ValueDecoder>
}
impl Ini {

	/* CONSTRUCTOR METHODS */

	/// Create a new ini from a file.
	/// Reads the file and parses the file immediately.
	pub fn from_file(file_path:&str) -> Result<Ini, Box<dyn Error>> {
		Ini::from_file_with_encoding(file_path, DEFAULT_DECODER, DEFAULT_ENCODER)
	}

	/// Create a new ini from a file and encoding settings.
	/// Reads the file and parses the file immediately.
	pub fn from_file_with_encoding<Encoder:ValueEncoder, Decoder:ValueDecoder>(file_path:&str, value_decoder:Decoder, value_encoder:Encoder) -> Result<Ini, Box<dyn Error>> {
		let file:FileRef = FileRef::new(file_path);
		Ok(
			Ini {
				categories: Self::parse_raw_contents(&file.read()?, &value_decoder),
				source_file: Some(file),
				value_encoder: Box::new(value_encoder),
				_value_decoder: Box::new(value_decoder)
			}
		)
	}

	/// Create a new ini from raw contents.
	/// Parses the contents immediately.
	pub fn from_contents(contents:&str) -> Ini {
		Ini::from_contents_with_encoding(contents, DEFAULT_DECODER, DEFAULT_ENCODER)
	}

	/// Create a new ini from raw contents and encoding settings..
	/// Parses the contents immediately.
	pub fn from_contents_with_encoding<Encoder:ValueEncoder, Decoder:ValueDecoder>(contents:&str, decoder:Decoder, encoder:Encoder) -> Ini {
		Ini {
			categories: Self::parse_raw_contents(&contents, &decoder),
			source_file: None,
			value_encoder: Box::new(encoder),
			_value_decoder: Box::new(decoder)
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
	pub fn get_category(&self, category_name:&str) -> Option<&IniCategory> {
		self.categories.iter().find(|category| category.name == category_name)
	}

	/// Try to find a mutable category in the ini.
	pub fn get_category_mut(&mut self, category_name:&str) -> Option<&mut IniCategory> {
		self.categories.iter_mut().find(|category| category.name == category_name)
	}

	/// Try to find a variable in the ini.
	pub fn get_variable(&self, category_name:&str, variable_name:&str) -> Option<&IniVariable> {
		self.get_category(category_name).and_then(|category| category.get_variable(variable_name))
	}

	/// Try to find a mutable variable in the ini.
	pub fn get_variable_mut(&mut self, category_name:&str, variable_name:&str) -> Option<&mut IniVariable> {
		self.get_category_mut(category_name).and_then(|category| category.get_variable_mut(variable_name))
	}



	/* PARSING METHODS */

	/// Parse raw contents into ini categories.
	fn parse_raw_contents(contents:&str, value_decoder:&dyn ValueEncoder) -> Vec<IniCategory> {
		const CATEGORY_OPEN:char = '[';
		const CATEGORY_CLOSE:char = ']';
		const VAR_SPLITTER:char = '=';
		
		let mut categories:Vec<IniCategory> = vec![IniCategory::new("")];
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
				categories.last_mut().unwrap().variables.push(IniVariable::new(var_name, &value_decoder(var_value)));
			}
		}
		categories.retain(|category| !category.variables.is_empty());
		categories
	}
}
impl Index<&str> for Ini {
	type Output = IniCategory;

	fn index(&self, category_name:&str) -> &Self::Output {
		static NONEXISTENT_INI_CATEGORY:IniCategory = IniCategory { name: String::new(), variables: Vec::new() };
		self.get_category(category_name).unwrap_or(&NONEXISTENT_INI_CATEGORY)
	}
}
impl IndexMut<&str> for Ini {
	fn index_mut(&mut self, category_name:&str) -> &mut Self::Output {
		match self.categories.iter().position(|category| category.name == category_name) {
			Some(index) => {
				&mut self.categories[index]
			},
			None => {
				self.categories.push(IniCategory::new(category_name));
				self.categories.last_mut().unwrap()
			}
		}
	}
}
impl ToString for Ini {
	fn to_string(&self) -> String {
		self.categories.iter().map(|category| category.to_encoded_string(&self.value_encoder)).collect::<Vec<String>>().join("\n\n")
	}
}



pub struct IniCategory {
	pub name:String,
	pub variables:Vec<IniVariable>
}
impl IniCategory {

	/* CONSTRUCTOR METHODS */

	/// Create a new category.
	pub fn new(name:&str) -> IniCategory {
		IniCategory {
			name: name.to_string(),
			variables: Vec::new()
		}
	}



	/* GETTER AND SETTER METHODS */

	/// Try to find a variable in the category.
	pub fn get_variable(&self, variable_name:&str) -> Option<&IniVariable> {
		self.variables.iter().find(|variable| variable.name == variable_name)
	}

	/// Try to find a mutable variable in the category.
	pub fn get_variable_mut(&mut self, variable_name:&str) -> Option<&mut IniVariable> {
		self.variables.iter_mut().find(|variable| variable.name == variable_name)
	}



	/* PARSING METHODS */

	/// Turn self into a string with encoded values.
	pub fn to_encoded_string(&self, value_encoder:&dyn ValueEncoder) -> String {
		format!("[{}]\n{}", self.name, self.variables.iter().map(|variable| variable.to_encoded_string(value_encoder)).collect::<Vec<String>>().join("\n"))
	}
}
impl Index<&str> for IniCategory {
	type Output = IniVariable;

	fn index(&self, variable_name:&str) -> &Self::Output {
		static NONEXISTENT_INI_VARIABLE:IniVariable = IniVariable { name: String::new(), value: String::new() };
		self.get_variable(variable_name).unwrap_or(&NONEXISTENT_INI_VARIABLE)
	}
}
impl IndexMut<&str> for IniCategory {
	fn index_mut(&mut self, variable_name:&str) -> &mut Self::Output {
		match self.variables.iter().position(|category| category.name == variable_name) {
			Some(index) => {
				&mut self.variables[index]
			},
			None => {
				self.variables.push(IniVariable::new(variable_name, ""));
				self.variables.last_mut().unwrap()
			}
		}
	}
}
impl ToString for IniCategory {
	fn to_string(&self) -> String {
		format!("[{}]\n{}", self.name, self.variables.iter().map(|variable| variable.to_string()).collect::<Vec<String>>().join("\n"))
	}
}



pub struct IniVariable {
	pub name:String,
	pub value:String
}
impl IniVariable {

	/* CONSTRUCTOR METHODS */

	/// Create a new variable.
	pub fn new(name:&str, value:&str) -> IniVariable {
		IniVariable {
			name: name.to_string(),
			value: value.to_string()
		}
	}



	/* PARSING METHODS */

	/// Turn self into a string with encoded values.
	pub fn to_encoded_string(&self, encoder:&dyn ValueEncoder) -> String {
		format!("{}={}", self.name, encoder(&self.value))
	}
}
impl ToString for IniVariable {
	fn to_string(&self) -> String {
		format!("{}={}", self.name, self.value)
	}
}