use super::*;

pub struct Iter<'a> {
    scanner: &'a Scanner<'a>,
}

impl Iterator for Iter<'_> {
    type Item = ScannerResult<Token>;

    fn next(&mut self) -> Option<ScannerResult<Token>> {
        todo!()
    }
}
