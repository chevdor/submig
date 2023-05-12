// use std::fmt;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SubmigError>;

#[derive(Error, Debug, Clone)]
pub enum SubmigError {
	#[error("The migration do not seem to be following the standard")]
	NonStandard,

	#[error("I/O Error")]
	IO,

	#[error("Parsing Error")]
	Parsing,

	#[error("Unknown error")]
	Unknown,
}
