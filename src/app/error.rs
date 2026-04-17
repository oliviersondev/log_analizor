use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AppError {
    Input(std::io::Error),
    Analyze(std::io::Error),
    Output(std::io::Error),
}

impl AppError {
    pub fn input(error: std::io::Error) -> Self {
        Self::Input(error)
    }

    pub fn analyze(error: std::io::Error) -> Self {
        Self::Analyze(error)
    }

    pub fn output(error: std::io::Error) -> Self {
        Self::Output(error)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input(error) => write!(f, "Input error: {error}"),
            Self::Analyze(error) => write!(f, "Analysis error: {error}"),
            Self::Output(error) => write!(f, "Output error: {error}"),
        }
    }
}

impl std::error::Error for AppError {}
