use std::{ error::Error, fmt::Debug };
use file_ref::FileRef;



const DEFAULT_NAME_DECODER:fn(&str) -> String = |v| v.trim().to_string();
const DEFAULT_NAME_ENCODER:fn(&String) -> String = |v| v.to_string();
const DEFAULT_VALUE_DECODER:fn(&str) -> String = |v| v.to_string();
const DEFAULT_VALUE_ENCODER:fn(&String) -> String = |v| v.to_string();
pub trait IniDecoder<ValueType>:Fn(&str) -> ValueType + Send + Sync + 'static {}
pub trait IniEncoder<ValueType>:Fn(&ValueType) -> String + Send + Sync + 'static {}
impl<ValueType, T:Fn(&str) -> ValueType + Send + Sync + 'static> IniDecoder<ValueType> for T {}
impl<ValueType, T:Fn(&ValueType) -> String + Send + Sync + 'static> IniEncoder<ValueType> for T {}



pub struct Ini<NameType:'static,ValueType:'static> {
	pub categories:Vec<IniCategory<NameType, ValueType>>,
	source_file:Option<FileRef>,
	_name_decoder:Box<dyn IniDecoder<NameType>>,
	name_encoder:Box<dyn IniEncoder<NameType>>,
	_value_decoder:Box<dyn IniDecoder<ValueType>>,
	value_encoder:Box<dyn IniEncoder<ValueType>>
}
impl<NameType, ValueType> Ini<NameType, ValueType> {

	/* CONSTRUCTOR METHODS */

	/// Create a new ini from a file.
	/// Reads the file and parses the file immediately.
	pub fn from_file(file_path:&str) -> Result<Ini<String, String>, Box<dyn Error>> {
		Ini::from_file_with_encoding(file_path, DEFAULT_NAME_DECODER, DEFAULT_NAME_ENCODER, DEFAULT_VALUE_DECODER, DEFAULT_VALUE_ENCODER)
	}

	/// Create a new ini from a file and encoding settings.
	/// Reads the file and parses the file immediately.
	pub fn from_file_with_encoding<NameDecoder:IniDecoder<NameType>, NameEncoder:IniEncoder<NameType>, ValueDecoder:IniDecoder<ValueType>, ValueEncoder:IniEncoder<ValueType>>(file_path:&str, name_decoder:NameDecoder, name_encoder:NameEncoder, value_decoder:ValueDecoder, value_encoder:ValueEncoder) -> Result<Self, Box<dyn Error>> {
		let file:FileRef = FileRef::new(file_path);
		Ok(
			Ini {
				categories: Self::parse_raw_contents(&file.read()?, &name_decoder, &value_decoder),
				source_file: Some(file),
				_name_decoder: Box::new(name_decoder),
				name_encoder: Box::new(name_encoder),
				_value_decoder: Box::new(value_decoder),
				value_encoder: Box::new(value_encoder)
			}
		)
	}

	/// Create a new ini from raw contents.
	/// Parses the contents immediately.
	pub fn from_contents(contents:&str) -> Ini<String, String> {
		Ini::from_contents_with_encoding(contents, DEFAULT_NAME_DECODER, DEFAULT_NAME_ENCODER, DEFAULT_VALUE_DECODER, DEFAULT_VALUE_ENCODER)
	}

	/// Create a new ini from raw contents and encoding settings..
	/// Parses the contents immediately.
	pub fn from_contents_with_encoding<NameDecoder:IniDecoder<NameType>, NameEncoder:IniEncoder<NameType>, ValueDecoder:IniDecoder<ValueType>, ValueEncoder:IniEncoder<ValueType>>(contents:&str, name_decoder:NameDecoder, name_encoder:NameEncoder, value_decoder:ValueDecoder, value_encoder:ValueEncoder) -> Self {
		Ini {
			categories: Self::parse_raw_contents(&contents, &name_decoder, &value_decoder),
			source_file: None,
				_name_decoder: Box::new(name_decoder),
				name_encoder: Box::new(name_encoder),
				_value_decoder: Box::new(value_decoder),
				value_encoder: Box::new(value_encoder)
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
	pub fn get_category(&self, category_name:&str) -> Option<&IniCategory<NameType, ValueType>> {
		self.categories.iter().find(|category| category.name == category_name)
	}

	/// Try to find a mutable category in the ini.
	pub fn get_category_mut(&mut self, category_name:&str) -> Option<&mut IniCategory<NameType, ValueType>> {
		self.categories.iter_mut().find(|category| category.name == category_name)
	}

	/// Create a category if it does not exist.
	pub fn set_category(&mut self, category_name:&str) {
		if !self.categories.iter().any(|category| category.name == category_name) {
			self.categories.push(IniCategory::new(category_name));
		}
	}

	/// Try to find a variable in the ini.
	pub fn get_variable<Name>(&self, category_name:&str, variable_name:Name) -> Option<&IniVariable<NameType, ValueType>> where NameType:PartialEq<Name> {
		self.get_category(category_name).and_then(|category| category.get_variable(variable_name))
	}

	/// Try to find a mutable variable in the ini.
	pub fn get_variable_mut<Name>(&mut self, category_name:&str, variable_name:Name) -> Option<&mut IniVariable<NameType, ValueType>> where NameType:PartialEq<Name> {
		self.get_category_mut(category_name).and_then(|category| category.get_variable_mut(variable_name))
	}

	/// Set the value of a variable.
	/// Creates the category and/or variable if it does not exist.
	pub fn set_variable<Name, Value>(&mut self, category_name:&str, variable_name:Name, variable_value:Value) where NameType:PartialEq<Name> + From<Name>, ValueType:From<Value> {
		match self.categories.iter().position(|category| category.name == category_name) {
			Some(index) => self.categories[index].set_variable(variable_name, variable_value),
			None => {
				let mut category = IniCategory::new(category_name);
				category.set_variable(variable_name, variable_value);
				self.categories.push(category);
			}
		}
	}



	/* PARSING METHODS */

	/// Parse raw contents into ini categories.
	fn parse_raw_contents(contents:&str, name_decoder:&dyn Fn(&str) -> NameType, value_decoder:&dyn Fn(&str) -> ValueType) -> Vec<IniCategory<NameType, ValueType>> {
		const CATEGORY_OPEN:char = '[';
		const CATEGORY_CLOSE:char = ']';
		const VAR_SPLITTER:char = '=';
		
		let mut categories:Vec<IniCategory<NameType, ValueType>> = vec![IniCategory::new("")];
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
				categories.last_mut().unwrap().variables.push(IniVariable::new(name_decoder(var_name), value_decoder(var_value)));
			}
		}
		categories.retain(|category| !category.variables.is_empty());
		categories
	}
}
impl<NameType, ValueType> ToString for Ini<NameType, ValueType> {
	fn to_string(&self) -> String {
		self.categories.iter().map(|category| category.to_encoded_string(&self.name_encoder, &self.value_encoder)).collect::<Vec<String>>().join("\n\n")
	}
}
impl<NameType:Debug, ValueType:Debug> Debug for Ini<NameType, ValueType> {
	fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.categories.iter().map(|category| format!("{:?}", category)).collect::<Vec<String>>().join("\n\n"))
	}
}



pub struct IniCategory<NameType, ValueType> {
	pub name:String,
	pub variables:Vec<IniVariable<NameType, ValueType>>
}
impl<NameType, ValueType> IniCategory<NameType, ValueType> {

	/* CONSTRUCTOR METHODS */

	/// Create a new category.
	pub fn new(name:&str) -> IniCategory<NameType, ValueType> {
		IniCategory {
			name: name.to_string(),
			variables: Vec::new()
		}
	}



	/* GETTER AND SETTER METHODS */

	/// Try to find a variable in the category.
	pub fn get_variable<Name>(&self, variable_name:Name) -> Option<&IniVariable<NameType, ValueType>> where NameType:PartialEq<Name> {
		self.variables.iter().find(|variable| variable.name == variable_name)
	}

	/// Try to find a mutable variable in the category.
	pub fn get_variable_mut<Name>(&mut self, variable_name:Name) -> Option<&mut IniVariable<NameType, ValueType>> where NameType:PartialEq<Name> {
		self.variables.iter_mut().find(|variable| variable.name == variable_name)
	}

	/// Set the value of a variable.
	/// Creates the variable if it does not exist.
	pub fn set_variable<Name, Value>(&mut self, variable_name:Name, variable_value:Value) where NameType:PartialEq<Name> + From<Name>, ValueType:From<Value> {
		match self.variables.iter().position(|variable| variable.name == variable_name) {
			Some(index) => self.variables[index].value = ValueType::from(variable_value),
			None => self.variables.push(IniVariable::new(variable_name, variable_value))
		}
	}



	/* PARSING METHODS */

	/// Turn self into a string with encoded values.
	pub fn to_encoded_string(&self, name_encoder:&dyn IniEncoder<NameType>, value_encoder:&dyn IniEncoder<ValueType>) -> String {
		format!("[{}]\n{}", self.name, self.variables.iter().map(|variable| variable.to_encoded_string(name_encoder, value_encoder)).collect::<Vec<String>>().join("\n"))
	}
}
impl<NameType:Debug, ValueType:Debug> Debug for IniCategory<NameType, ValueType> {
	fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "[{}]\n{}", self.name, self.variables.iter().map(|variable| format!("{:?}", variable)).collect::<Vec<String>>().join("\n"))
	}
}



pub struct IniVariable<NameType, ValueType> {
	pub name:NameType,
	pub value:ValueType
}
impl<NameType, ValueType> IniVariable<NameType, ValueType> {

	/* CONSTRUCTOR METHODS */

	/// Create a new variable.
	pub fn new<Name, Value>(name:Name, value:Value) -> Self where NameType:From<Name>, ValueType:From<Value> {
		IniVariable {
			name: NameType::from(name),
			value: ValueType::from(value)
		}
	}



	/* PARSING METHODS */

	/// Turn self into a string with encoded values.
	pub fn to_encoded_string(&self, name_encoder:&dyn IniEncoder<NameType>, value_encoder:&dyn IniEncoder<ValueType>) -> String {
		format!("{}={}", name_encoder(&self.name), value_encoder(&self.value))
	}
}
impl<NameType:Debug, ValueType:Debug> Debug for IniVariable<NameType, ValueType> {
	fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}={:?}", self.name, self.value)
	}
}