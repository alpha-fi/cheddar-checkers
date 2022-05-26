use crate::board::BoardPosition;

#[derive(Debug, PartialEq, Eq)]
pub enum InputError {
	TooFewTokens,
	InvalidTokens { tokens : Vec<TokenError> }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenError {
	MissingFile { token : String },
	MissingRank { token : String },
	ZeroRank { token : String },
	InvalidCharacter { token : String, char_index : usize }
}

///
/// Parse a move from a string
///
pub fn parse_move(the_move : &str) -> Result<Vec<BoardPosition>, InputError> {
	let results : Vec<_> = the_move.split_whitespace()
		.map(token_validator)
		.collect();

	let (ok_iter, err_iter) : (Vec<_>, Vec<_>) = results.into_iter()
		.map(
			|result|
				match result {
					Ok(v) => (Some(v), None),
					Err(e) => (None, Some(e))
				})
		.unzip();

	let errors : Vec<_> = err_iter.into_iter()
		.filter_map(|error| error)
		.collect();

	if !errors.is_empty() {
		return Err(InputError::InvalidTokens { tokens : errors });
	}

	let positions : Vec<_> = ok_iter.into_iter()
		.filter_map(|position| position)
		.collect();

	if positions.len() < 2 {
		return Err(InputError::TooFewTokens);
	}

	Ok(positions)
}

//
// Determines whether a position string is valid.
// Expects a strict sequence of alphabetic characters (rank)
// followed by a sequence of numeric characters (file).
//
fn token_validator(token : &str) -> Result<BoardPosition, TokenError> {
	let parse_file_rank_result = parse_file_rank(token);
	match parse_file_rank_result {
		Ok((file, rank)) => {
			if file.is_empty() {
				return Err(TokenError::MissingFile { token : token.to_string() });
			}
			if rank.is_empty() {
				return Err(TokenError::MissingRank { token : token.to_string() });
			}

			let row : usize = rank.parse::<usize>().unwrap();
			let col : usize = file_to_row_position(&file);

			if row == 0 {
				return Err(TokenError::ZeroRank { token : token.to_string() });
			}

			Ok(BoardPosition::new(row - 1, col - 1))
		},
		Err(e) => Err(e)
	}
}

enum ParseState {
	File,
	Rank
}

//
// Parse a string and return a tuple containing
// the file and rank, respectively
//
fn parse_file_rank(token : &str) -> Result<(String, String), TokenError> {
	let mut file : String = String::new();
	let mut rank : String = String::new();

	let mut iter = token.chars().enumerate();
	let mut char_opt = iter.next();
	let mut parse_state = ParseState::File;

	while char_opt.is_some() {
		let (index, ch) = char_opt.unwrap();
		match parse_state {
			ParseState::File => {
				if ch.is_alphabetic() {
					file.push(ch);
					char_opt = iter.next();
				} else if ch.is_numeric() {
					parse_state = ParseState::Rank;
				} else {
					return Err(TokenError::InvalidCharacter {
						token : token.to_string(), char_index : index });
				}
			}
			ParseState::Rank => {
				if ch.is_numeric() {
					rank.push(ch);
					char_opt = iter.next();
				} else {
					return Err(TokenError::InvalidCharacter {
						token : token.to_string(), char_index : index });
				}
			}
		}
	}

	Ok((file, rank))
}

//
// Convert string of alphabetic characters to an index
//
fn file_to_row_position(file : &str) -> usize {
	let mut row : usize = 0;
	let alphabet_length = 26;

	for c in file.chars() {
		row = row * alphabet_length + char_to_position(c);
	}

	row
}

//
// Convert a single alphabetic character to number
// Case insensitive [a-z] -> [1-26]
//
fn char_to_position( c : char ) -> usize {
	debug_assert!(c.is_alphabetic());

	match c {
		'A'..='Z' => (c as usize) - ('A' as usize) + 1,
		'a'..='z' => (c as usize) - ('a' as usize) + 1,
		_ => unreachable!()
	}
}