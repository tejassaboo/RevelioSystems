use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum AuthenticateError {
	SignatureFormattingError,
	InvalidSignature,
	Expired,
	LongValidity,
	NonceReuse,
	InvalidMessage,
}

impl Display for AuthenticateError {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		write!(
			f,
			"{}",
			match self {
				AuthenticateError::SignatureFormattingError => "The signature must be base64 encoded.",
				AuthenticateError::InvalidSignature => "The signature is invalid.",
				AuthenticateError::Expired => "The request is expired.",
				AuthenticateError::LongValidity => "The request expires too far in the future.",
				AuthenticateError::NonceReuse => "This nonce was used before.",
				AuthenticateError::InvalidMessage => "The message is not formatted correctly.",
			},
		)
	}
}

impl std::error::Error for AuthenticateError {}


#[derive(Debug)]
pub enum HMACKeyError {
	KeyLengthMismatch,
}

impl Display for HMACKeyError {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		write!(
			f,
			"{}",
			match self {
				HMACKeyError::KeyLengthMismatch => "Provided key has the wrong length.",
			}
		)
	}
}

impl Error for HMACKeyError {}

#[derive(Debug)]
pub enum SodiumOxideError {
	InitError,
}

impl Display for SodiumOxideError {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		write!(
			f,
			"{}",
			match self {
				SodiumOxideError::InitError => "Sodium Oxide failed to initialize.",
			}
		)
	}
}

impl Error for SodiumOxideError {}