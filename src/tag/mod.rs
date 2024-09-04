use std::{
	collections::HashMap,
	str::{Chars, FromStr},
};

use chrono::NaiveDate;
use ui::TagWidget;

mod ui;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Tag {
	name: String,
	value: Option<TagValue>,

	#[serde(skip)]
	editing_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TagValue {
	Int(i64),
	Float(f64),
	Date(NaiveDate),
	Text(String),
	List(Vec<TagValue>),
	Dictionary(HashMap<String, TagValue>),
	Tag(Box<Tag>),
}

impl Default for Tag {
	fn default() -> Self {
		Self {
			name: String::from("new_tag"),
			value: None,
			editing_text: None,
		}
	}
}

/// Do not use with Cycle iterators or endless iterators of whitespace, will loop infinitely!
#[derive(Debug, Clone)]
struct TagStringChars<'a> {
	chars: Chars<'a>,
	peeked: Vec<Option<char>>,
}

impl Iterator for TagStringChars<'_> {
	type Item = char;

	fn next(&mut self) -> Option<Self::Item> {
		match self.next_with_whitespace() {
			Some(whitespace) if whitespace.is_whitespace() => self.next(),
			Some(other) => Some(other),
			None => None,
		}
	}
}

impl<'a> TagStringChars<'a> {
	pub fn new(s: &'a str) -> Self {
		Self {
			chars: s.chars(),
			peeked: Vec::new(),
		}
	}

	pub fn peek(&mut self) -> Option<char> {
		if let Some(Some(c)) = self.peeked.iter().find(|c| {
			if let Some(c) = c {
				!c.is_whitespace()
			} else {
				true
			}
		}) {
			Some(*c)
		} else {
			while let Some(c) = self.chars.next() {
				self.peeked.push(Some(c));

				if !c.is_whitespace() {
					return Some(c);
				}
			}

			None
		}
	}

	pub fn next_with_whitespace(&mut self) -> Option<char> {
		if self.peeked.is_empty() {
			self.chars.next()
		} else {
			self.peeked.remove(0)
		}
	}

	pub fn peek_with_whitespace(&mut self) -> Option<char> {
		if self.peeked.is_empty() {
			self.peeked.push(self.chars.next());
		}

		self.peeked[0]
	}
}

impl FromStr for Tag {
	type Err = TagError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::parse(&mut TagStringChars::new(s))
	}
}

impl Tag {
	fn parse(chars: &mut TagStringChars) -> Result<Self, TagError> {
		let first_char = chars.next().ok_or(TagError::EmptyTag)?;
		let mut name = String::new();

		if !(first_char.is_alphabetic() || first_char == '_') {
			return Err(TagError::InvalidFirstChar(first_char));
		}

		name.push(first_char);

		let mut value = None;

		while let Some(c) = chars.peek() {
			match c {
				'(' => {
					chars.next().expect("peek returned some");

					value = Some(TagValue::parse(chars)?);

					if chars.next().unwrap_or(' ') != ')' {
						return Err(TagError::UnclosedParen);
					}
				}
				alphanum_ if alphanum_.is_alphanumeric() || alphanum_ == '_' => {
					chars.next().expect("peek returned some");
					name.push(alphanum_);
				}
				_ => {
					break;
				}
			}
		}

		Ok(Self {
			name,
			value,
			editing_text: None,
		})
	}

	pub fn widget(&mut self, edit_mode: bool) -> TagWidget {
		TagWidget::new(self, edit_mode)
	}

	pub fn get_editing_text(&mut self) -> &mut String {
		match self.editing_text {
			Some(ref mut text) => text,
			None => {
				self.editing_text = Some(self.to_string());
				self.editing_text.as_mut().unwrap()
			}
		}
	}

	pub fn apply_text(&mut self) -> Result<(), TagError> {
		if let Some(text) = self.editing_text.take() {
			*self = text.parse()?;
		}

		Ok(())
	}
}

impl FromStr for TagValue {
	type Err = TagError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::parse(&mut TagStringChars::new(s))
	}
}

impl TagValue {
	fn parse(chars: &mut TagStringChars) -> Result<Self, TagError> {
		let first_char = chars.peek().ok_or(TagError::EmptyTagValue)?;

		match first_char {
			'"' | '\'' => Ok(Self::Text(
				Self::parse_string(chars).expect("peek returned \" or '")?,
			)),
			'0'..='9' | '.' | '-' => {
				let mut s: String = String::new();

				let mut started = false;

				while let Some(c) = chars.peek_with_whitespace() {
					match c {
						'0'..='9' | '.' | '-' => {
							chars.next_with_whitespace().expect("peek returned some");
							s.push(c);
							started = true;
						}
						' ' if !started => {
							chars.next_with_whitespace().expect("peek returned some");
						}
						_ => {
							break;
						}
					}
				}

				if let Ok(date) = s.parse::<NaiveDate>() {
					Ok(Self::Date(date))
				} else {
					if let Ok(int) = s.parse::<i64>() {
						Ok(Self::Int(int))
					} else {
						match s.parse::<f64>() {
							Ok(float) => Ok(Self::Float(float)),
							Err(e) => Err(TagError::CantParseNumber(e)),
						}
					}
				}
			}
			'[' => {
				chars.next().expect("peek returned some");

				let mut elements = Vec::new();
				let mut ready_for_element = true;
				let mut closed = false;

				while let Some(c) = chars.peek() {
					match c {
						']' => {
							chars.next().expect("Peek returned some");
							closed = true;
							break;
						}
						',' => {
							chars.next().expect("Peek returned some");
							if ready_for_element {
								return Err(TagError::RepeatedComma);
							} else {
								ready_for_element = true
							}
						}
						_ => {
							if !ready_for_element {
								return Err(TagError::MissingComma);
							}

							elements.push(TagValue::parse(chars)?);
							ready_for_element = false;
						}
					}
				}

				if !closed {
					return Err(TagError::UnclosedList);
				}

				Ok(Self::List(elements))
			}
			'{' => {
				chars.next().expect("peek returned some");

				let mut check_comma = false;

				let mut parse_element =
					|chars: &mut TagStringChars| -> Option<Result<(String, TagValue), TagError>> {
						if check_comma {
							if chars.peek().unwrap_or(' ') != ',' {
								return None;
							}

							chars.next().expect("peek returned some");
						}

						let name = match Self::parse_string(chars)? {
							Ok(name) => name,
							Err(e) => return Some(Err(e)),
						};

						if chars.next().unwrap_or(' ') != ':' {
							return Some(Err(TagError::InvalidDictonaryElement));
						}

						let value = match TagValue::parse(chars) {
							Ok(value) => value,
							Err(e) => return Some(Err(e)),
						};

						check_comma = true;

						Some(Ok((name, value)))
					};

				let mut result: HashMap<String, TagValue> = HashMap::new();

				while let Some(element) = parse_element(chars) {
					let (key, value) = element?;
					result.insert(key, value);
				}

				if chars.next().unwrap_or(' ') != '}' {
					return Err(TagError::UnclosedDictionary);
				}

				Ok(Self::Dictionary(result))
			}
			c if c.is_alphabetic() || c == '_' => Ok(Self::Tag(Box::new(Tag::parse(chars)?))),
			other => Err(TagError::InvalidFirstValueChar(other)),
		}
	}
}

impl TagValue {
	fn parse_string(chars: &mut TagStringChars) -> Option<Result<String, TagError>> {
		let first_char = chars.peek()?;

		match first_char {
			'"' | '\'' => {}
			_ => return None,
		}

		chars.next().expect("peek returned some");

		let mut string = String::new();

		while let Some(c) = chars.next_with_whitespace() {
			match c {
				end if end == first_char => {
					break;
				}
				'\\' => match match chars.next().ok_or(TagError::UnclosedString) {
					Ok(next) => next,
					Err(e) => return Some(Err(e)),
				} {
					'n' => string.push('\n'),
					'r' => string.push('\r'),
					't' => string.push('\t'),
					'\\' => string.push('\\'),
					end if end == first_char => string.push(end),
					other => {
						return Some(Err(TagError::BadEscapeChar(other)));
					}
				},
				other => {
					string.push(other);
				}
			}
		}

		Some(Ok(string))
	}
}

impl ToString for Tag {
	fn to_string(&self) -> String {
		let mut result = self.name.clone();

		if let Some(value) = &self.value {
			result.push('(');
			result.push_str(value.to_string().as_str());
			result.push(')');
		}

		result
	}
}

impl ToString for TagValue {
	fn to_string(&self) -> String {
		match self {
			Self::Text(text) => format!("\"{}\"", text),
			Self::Int(int) => int.to_string(),
			Self::Float(float) => float.to_string(),
			Self::Date(date) => date.to_string(),
			Self::List(list) => {
				let mut inner = list
					.iter()
					.map(|el| format!("{}, ", el.to_string()))
					.collect::<String>();

				if !inner.is_empty() {
					inner.pop();
					inner.pop();
				}

				format!("[{}]", inner)
			}
			Self::Dictionary(dict) => {
				let mut inner = dict
					.iter()
					.map(|(k, v)| format!("\"{}\": {}, ", k, v.to_string()))
					.collect::<String>();

				if !inner.is_empty() {
					inner.pop();
					inner.pop();
				}

				format!("{{{}}}", inner)
			}
			Self::Tag(tag) => tag.to_string(),
		}
	}
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum TagError {
	#[error("Tag is empty")]
	EmptyTag,

	#[error("Tag value is empty")]
	EmptyTagValue,

	#[error("Tag name must begin with a letter or _, but here started with `{0}`")]
	InvalidFirstChar(char),

	#[error("Tag value must begin with a letter or _ for tags, a number, or \"/' for text, but here started with `{0}`")]
	InvalidFirstValueChar(char),

	#[error("Unclosed parenthesis")]
	UnclosedParen,

	#[error("\"/' not closed in tag value")]
	UnclosedString,

	#[error("Couldn't parse number: {0}")]
	CantParseNumber(std::num::ParseFloatError),

	#[error("List square brackets are not closed")]
	UnclosedList,

	#[error("Dictionary curly braces are not closed")]
	UnclosedDictionary,

	#[error("A dictonary should be made of quote surrounded strings, followed by `:`, followed by another value")]
	InvalidDictonaryElement,

	#[error("Invalid escaped character `\\{0}`. Use `\\\\` to make a `\\`")]
	BadEscapeChar(char),

	#[error("More than one comma in list/dictionary")]
	RepeatedComma,

	#[error("Missing comma in list/dictionary")]
	MissingComma,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn de_ser() {
		let base = [
			"some_funny_tag(-45.3)",
			"_The_Tag(OtherTag(\"lol\"))",
			"emptytag",
			"empty_list_value([])",
			"list([\"haha\", 4.5])",
			"nested([[\"lol\", 23], OtherTag([])])",
		];

		let dict_test = "dict({\"lol\": 4, \"haha\": inner_tag(4.53)})";
		let dict_test_alt = "dict({\"haha\": inner_tag(4.53), \"lol\": 4})";

		let new: Vec<String> = base
			.clone()
			.into_iter()
			.map(|b| b.parse::<Tag>().unwrap())
			.map(|p| p.to_string())
			.collect();

		let new_dict = dict_test.parse::<Tag>().unwrap().to_string();

		base.into_iter()
			.zip(new)
			.for_each(|(a, b)| assert_eq!(a, b.as_str()));

		assert!(new_dict == dict_test || new_dict == dict_test_alt);
	}

	#[test]
	fn errors() {
		let base = [
			"3InvalidTag",
			"Unclosed(",
			"InvalidValue(')",
			"UnclosedList([)",
			"MoreCommas([43, , 5.4])",
			"MissingComma([\"hello\" 54])",
		];

		let errors: Vec<TagError> = base
			.into_iter()
			.map(|b| b.parse::<Tag>().unwrap_err())
			.collect();

		let required_errors = [
			TagError::InvalidFirstChar('3'),
			TagError::EmptyTagValue,
			TagError::UnclosedParen,
			TagError::InvalidFirstValueChar(')'),
			TagError::RepeatedComma,
			TagError::MissingComma,
		];

		errors
			.into_iter()
			.zip(required_errors)
			.for_each(|(a, b)| assert_eq!(a, b));
	}
}
