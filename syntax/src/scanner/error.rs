#[derive(Debug)]
pub struct ScannerError {
    pub error_type: ErrorType,
    pub line: usize,
}

#[derive(Debug)]
pub enum ErrorType {
    NotUtf8,
    UnknownByte(u8),
    UnterminatedStringLiteral,
}

impl std::fmt::Display for ScannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self.error_type {
            ErrorType::NotUtf8 => String::from("String is not a valid UTF-8 sequence"),
            ErrorType::UnknownByte(a) => format!("Byte {a} is unknown"),
            ErrorType::UnterminatedStringLiteral => String::from("Unterminated string literal"),
        };

        write!(f, "[line {}]: {message}", self.line)
    }
}
