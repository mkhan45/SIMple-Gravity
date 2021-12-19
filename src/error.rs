use std::error::Error;

#[derive(Debug)]
pub enum SimError { }

impl std::fmt::Display for SimError {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        write!(formatter, "SimError")
    }
}

impl Error for SimError {}
