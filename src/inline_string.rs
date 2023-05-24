use std::fmt;

const LEN: usize = 16usize;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(align(8))]
pub struct InlineString {
    data: [u8; LEN],
}

impl InlineString {
    pub fn new(s: &[u8]) -> Self {
        assert!(
            s.len() <= LEN,
            "The length of the string \"{}\" must be less than or equal to {LEN}",
            std::str::from_utf8(s).unwrap()
        );

        let mut data = [0u8; LEN];
        s.iter().enumerate().for_each(|(i, &b)| data[i] = b);

        Self { data }
    }

    pub fn bytes<'a>(&'a self) -> impl Iterator<Item = u8> + 'a {
        self.data[..self.len()].iter().cloned()
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.data[..self.len()]).unwrap()
    }

    pub fn len(&self) -> usize {
        let mut len = 0;
        while len < LEN && self.data[len] != 0 {
            len += 1;
        }
        len
    }
}

impl fmt::Debug for InlineString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\"{}\"",
            std::str::from_utf8(&self.data[..self.len()]).unwrap()
        )
    }
}

impl fmt::Display for InlineString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            std::str::from_utf8(&self.data[..self.len()]).unwrap()
        )
    }
}
