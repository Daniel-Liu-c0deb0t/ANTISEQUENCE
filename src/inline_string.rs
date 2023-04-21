use std::fmt;

const LEN: usize = 16usize;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(align(8))]
pub struct InlineString {
    data: [u8; LEN],
}

impl InlineString {
    pub fn new(s: &str) -> Self {
        assert!(s.len() <= LEN);

        let mut data = [0u8; LEN];
        s.bytes().enumerate().for_each(|(i, b)| data[i] = b);

        Self { data }
    }

    pub fn bytes<'a>(&'a self) -> impl Iterator<Item = u8> + 'a {
        self.data.iter().cloned()
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
        use std::fmt::Write;
        f.write_char('"')?;
        f.write_str(std::str::from_utf8(&self.data[..self.data.len()]).unwrap())?;
        f.write_char('"')
    }
}

impl fmt::Display for InlineString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(std::str::from_utf8(&self.data[..self.data.len()]).unwrap())
    }
}
