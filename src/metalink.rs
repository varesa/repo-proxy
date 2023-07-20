use crate::xml_tree::XmlTree;

#[derive(Clone, Debug)]
pub struct Metalink {}

impl Metalink {
    pub fn try_from_bytes(input: &[u8]) -> Result<Self, anyhow::Error> {
        let tree = XmlTree::try_from_bytes(input)?;
        println!("{tree:#?}");

        Ok(Metalink {})
    }
}