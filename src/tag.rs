use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Tag {
	name: String,
	value: Option<TagValue>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TagValue {
	Int(i64),
	Float(f64),
	Text(String),
	Tag(Box<Tag>),
}

impl FromStr for Tag {
	type Err = TagError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut char_indicies = s.char_indices();
		let first_char = char_indicies.next().ok_or(TagError::EmptyTag)?.1;

		if !(first_char.is_alphabetic() || first_char == '_') {
			return Err(TagError::InvalidFirstChar(first_char));
		}

		let mut paren_count: i32 = 0;
		let mut value_delim: Option<(usize, usize)> = None;

		for (i, c) in char_indicies {
			if !(c.is_ascii_alphanumeric() || c == '_') {
				if c == '(' {
					paren_count += 1;

					if value_delim.is_none() {
						value_delim = Some((i + 1, 0));
					}
				} else if c == ')' {
					paren_count -= 1;

					if let Some(value_delim) = &mut value_delim {
						if paren_count == 0 {
							*value_delim = (value_delim.0, i);
						}
					}
				}
			}
		}

		if paren_count > 0 {
			return Err(TagError::UnclosedParen);
		} else if paren_count < 0 {
			return Err(TagError::TooManyClosedParen);
		}

		let value = match value_delim.map(|(start, end)| TagValue::from_str(&s[start..end])) {
			Some(result) => Some(result?),
			None => None,
		};

		Ok(Self {
			name: s[0..(if let Some((start, _)) = value_delim {
				start - 1
			} else {
				s.len()
			})]
				.to_owned(),
			value,
		})
	}
}

impl FromStr for TagValue {
	type Err = TagError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut chars = s.chars();

		let first_char = chars.next().ok_or(TagError::EmptyTagValue)?;

		match first_char {
			'"' | '\'' => {
				if first_char != chars.last().unwrap_or(' ') {
					Err(TagError::UnclosedString)
				} else {
					Ok(Self::Text(String::from(&s[1..(s.len() - 1)])))
				}
			}
			c if c.is_numeric() || c == '.' || c == '-' => {
				if let Ok(int) = s.parse::<i64>() {
					Ok(Self::Int(int))
				} else {
					match s.parse::<f64>() {
						Ok(float) => Ok(Self::Float(float)),
						Err(e) => Err(TagError::CantParseNumber(e)),
					}
				}
			}
			c if c.is_alphabetic() || c == '_' => Ok(Self::Tag(Box::new(s.parse()?))),
			other => Err(TagError::InvalidFirstValueChar(other)),
		}
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

	#[error("Tag name must begin with a letter or _, but here started with {0}")]
	InvalidFirstChar(char),

	#[error("Tag value must begin with a letter or _ for tags, a number, or \"/' for text, but here started with {0}")]
	InvalidFirstValueChar(char),

	#[error("Unclosed parenthesis")]
	UnclosedParen,

	#[error("Too many closed parentheses")]
	TooManyClosedParen,

	#[error("\"/' not closed in tag value")]
	UnclosedString,

	#[error("Couldn't parse number: {0}")]
	CantParseNumber(std::num::ParseFloatError),
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
		];

		let new: Vec<String> = base
			.clone()
			.into_iter()
			.map(|b| b.parse::<Tag>().unwrap())
			.map(|p| p.to_string())
			.collect();

		assert!(base.into_iter().zip(new).all(|(a, b)| a == b.as_str()));
	}

	#[test]
	fn errors() {
		let base = ["3InvalidTag", "Unclosed(", "InvalidValue(')"];

		let errors: Vec<TagError> = base
			.into_iter()
			.map(|b| b.parse::<Tag>().unwrap_err())
			.collect();

		let required_errors = [
			TagError::InvalidFirstChar('3'),
			TagError::UnclosedParen,
			TagError::UnclosedString,
		];

		assert!(errors.into_iter().zip(required_errors).all(|(a, b)| a == b));
	}
}
