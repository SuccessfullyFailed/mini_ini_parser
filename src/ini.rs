use std::{ error::Error, ops::{ Index, IndexMut } };
use file_ref::FileRef;



const CATEGORY_CHARS:[char; 2] = ['[', ']'];
const VARIABLE_SPLIT_CHAR:char = '=';
const UNAVAILABLE_PROPERTY_PLACEHOLDER:&str = "UNAVAILABLE_PROPERTY_ERROR";
pub type IniValueEncoder = &'static dyn Fn(&str) -> String;
pub type IniValueDecoder = &'static dyn Fn(&str) -> String;



#[derive(Clone)]
pub struct IniCore {
	data:Vec<IniCategory>,
	encoder:IniValueEncoder,
	decoder:IniValueDecoder,
	source_file:Option<FileRef>
}
impl IniCore {

	/* CONSTRUCTOR METHODS */

	/// Create a new IniCore from a file.
	pub fn from_file(path:&str, encoder:IniValueEncoder, decoder:IniValueDecoder) -> Result<IniCore, Box<dyn Error>> {
		let source_file:FileRef = FileRef::new(path);
		Ok(IniCore {
			data: Self::parse_contents(&source_file.read()?, decoder)?,
			encoder,
			decoder,
			source_file: Some(source_file)
		})
	}

	/// Create a new IniCore from contents.
	pub fn from_contents(contents:&str, encoder:IniValueEncoder, decoder:IniValueDecoder) -> Result<IniCore, Box<dyn Error>> {
		Ok(IniCore {
			data: Self::parse_contents(contents, decoder)?,
			encoder,
			decoder,
			source_file: None
		})
	}



	/* PROPERTY GETTER METHODS */

	/// Get all categories.
	pub fn categories(&self) -> &Vec<IniCategory> {
		&self.data
	}

	/// Get a mutable reference all categories.
	pub fn categories_mut(&mut self) -> &mut Vec<IniCategory> {
		&mut self.data
	}



	/* USAGE METHODS */

	/// Parse some contents to IniCategories.
	fn parse_contents(contents:&str, decoder:IniValueDecoder) -> Result<Vec<IniCategory>, Box<dyn Error>> {

		// Loop through lines in contents.
		let mut categories:Vec<IniCategory> = Vec::new();
		let mut variables:Vec<IniVariable> = Vec::new();
		let mut current_category_name:String = String::new();
		for line in contents.replace('\r', "\n").split('\n') {
			let line:&str = line.trim();

			// Empty line.
			if line.is_empty() {
				continue;
			}

			// New category.
			else if line.starts_with(CATEGORY_CHARS[0]) && line.ends_with(CATEGORY_CHARS[1]) {
				if !variables.is_empty() {
					categories.push(IniCategory::new(&current_category_name, variables));
				}
				current_category_name = line[1..line.len() - 1].trim().to_string();
				variables = Vec::new();
			}
			
			// New variable.
			else if line.contains(VARIABLE_SPLIT_CHAR) {
				let raw_var_name:&str = line.split(VARIABLE_SPLIT_CHAR).next().unwrap();
				let name:&str = raw_var_name.trim();
				let value:&str = &line[raw_var_name.len() + VARIABLE_SPLIT_CHAR.len_utf8()..];
				variables.push(IniVariable::new(name, &decoder(value)));
			}

			// No matches.
			else {
				return Err(format!("Could not parse IniCore. Unsupported line: '{line}'").into());
			}
		}

		// Add trailing data.
		if !variables.is_empty() {
			categories.push(IniCategory::new(&current_category_name, variables));
		}

		// Return parsed categories.
		Ok(categories)
	}

	/// Encode values and parse to string.
	pub(super) fn to_string_encoded_values(&self) -> String {
		self.data.iter().filter(|category| category.is_ok()).map(|category| category.to_string_encoded(self.encoder, self.decoder)).collect::<Vec<String>>().join("\n\n")
	}

	/// Save the changes made to the original file if there is one.
	pub fn save_changes(&self) -> Result<(), Box<dyn Error>> {
		match &self.source_file {
			Some(file) => file.write(self.to_string_encoded_values()),
			None => Err("Could not save changes to midi, midi did not come from a file. Please use 'save_to_file' instead.".into())
		}
	}

	/// Save the ini to the specified file.
	pub fn save_to_file(&self, file_path:&str) -> Result<(), Box<dyn Error>> {
		FileRef::new(file_path).write(self.to_string_encoded_values())
	}
}
impl Index<&str> for IniCore {
	type Output = IniCategory;
	fn index(&self, target_name:&str) -> &Self::Output {
		match self.data.iter().position(|variable| &variable.name == target_name) {
			Some(index) => &self.data[index],
			None => IniCategory::error_instance()
		}
	}
}
impl IndexMut<&str> for IniCore {
	fn index_mut(&mut self, target_name:&str) -> &mut Self::Output {
		match self.data.iter().position(|category| &category.name == target_name) {
			Some(index) => &mut self.data[index],
			None => {
				self.data.push(IniCategory { name: target_name.to_string(), data: Vec::new() });
				self.data.iter_mut().last().unwrap()
			}
		}
	}
}



#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IniCategory {
	pub name:String,
	pub data:Vec<IniVariable>
}
impl IniCategory {

	/* CONSTRUCTOR METHODS */

	/// Create a new empty IniCategory.
	pub fn empty(name:&str) -> IniCategory {
		IniCategory::new(name, Vec::new())
	}

	/// Create a new IniCategory.
	pub fn new(name:&str, variables:Vec<IniVariable>) -> IniCategory {
		if name == UNAVAILABLE_PROPERTY_PLACEHOLDER {
			panic!("'{UNAVAILABLE_PROPERTY_PLACEHOLDER}' should not be used for an ini category name. This string is reserved for incorrect property fetching.");
		}
		IniCategory {
			name: name.to_string(),
			data: variables
		}
	}

	/// Get the error IniCategory. Used for when an ini category is expected, but not available.
	#[allow(static_mut_refs)]
	pub fn error_instance() -> &'static IniCategory {
		unsafe {
			static mut INVALID_RETURN:Option<IniCategory> = None;
			match INVALID_RETURN.as_ref() {
				Some(ini_var) => ini_var,
				None => {
					INVALID_RETURN = Some(IniCategory { name: String::from(UNAVAILABLE_PROPERTY_PLACEHOLDER), data: Vec::new()});
					Self::error_instance()
				}
			}
		}
	}



	/* PROPERTY GETTER METHODS */

	/// Check if the gotten variable is ok, not the error category.
	pub fn is_ok(&self) -> bool {
		self.name != UNAVAILABLE_PROPERTY_PLACEHOLDER && !self.data.is_empty()
	}

	/// Encode values and parse to string.
	pub(super) fn to_string_encoded(&self, encoder:IniValueEncoder, decoder:IniValueDecoder) -> String {
		format!("{}{}{}\n{}", CATEGORY_CHARS[0], self.name, CATEGORY_CHARS[1], self.data.iter().filter(|variable| variable.is_ok()).map(|variable| variable.to_string_encoded(encoder, decoder)).collect::<Vec<String>>().join("\n"))
	}
}
impl Index<&str> for IniCategory {
	type Output = IniVariable;
	fn index(&self, target_name:&str) -> &Self::Output {
		match self.data.iter().position(|variable| &variable.name == target_name) {
			Some(index) => &self.data[index],
			None => IniVariable::error_instance()
		}
	}
}
impl IndexMut<&str> for IniCategory {
	fn index_mut(&mut self, target_name:&str) -> &mut Self::Output {
		match self.data.iter().position(|variable| &variable.name == target_name) {
			Some(index) => &mut self.data[index],
			None => {
				self.data.push(IniVariable::new(target_name, ""));
				self.data.iter_mut().last().unwrap()
			}
		}
	}
}



#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IniVariable {
	pub name:String,
	pub value:String
}
impl IniVariable {
	
	/* CONSTRUCTOR METHODS */

	/// Create a new IniVariable.
	pub fn new(name:&str, value:&str) -> IniVariable {
		if name == UNAVAILABLE_PROPERTY_PLACEHOLDER {
			panic!("'{UNAVAILABLE_PROPERTY_PLACEHOLDER}' should not be used for an ini variable name. This string is reserved for incorrect property fetching.");
		}
		IniVariable {
			name: name.to_string(),
			value: value.to_string()
		}
	}

	/// Get the error IniVariable. Used for when an ini variable is expected, but not available.
	#[allow(static_mut_refs)]
	pub fn error_instance() -> &'static IniVariable {
		unsafe {
			static mut INVALID_RETURN:Option<IniVariable> = None;
			match INVALID_RETURN.as_ref() {
				Some(ini_var) => ini_var,
				None => {
					INVALID_RETURN = Some(IniVariable { name: String::from(UNAVAILABLE_PROPERTY_PLACEHOLDER), value: String::from(UNAVAILABLE_PROPERTY_PLACEHOLDER) });
					Self::error_instance()
				}
			}
		}
	}



	/* PROPERTY GETTER METHODS */

	/// Check if the gotten variable is ok, not the error variable.
	pub fn is_ok(&self) -> bool {
		self.name != UNAVAILABLE_PROPERTY_PLACEHOLDER && !self.value.is_empty()
	}

	/// Encode values and parse to string.
	pub(super) fn to_string_encoded(&self, encoder:IniValueEncoder, decoder:IniValueDecoder) -> String {
		format!("{}{}{}", self.name, VARIABLE_SPLIT_CHAR, encoder(&decoder(&self.value)))
	}
}