
#[derive(Clone, Debug)]
pub struct Metalink {}

impl Metalink {
    pub fn try_from_string(input: &[u8]) -> Self {
        let parser = xml::reader::EventReader::new(input);
        for e in parser {
            println!("{e:?}");
        }

        Metalink {}
    }
}