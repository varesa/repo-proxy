use crate::xml_tree::XmlTree;

#[derive(Clone, Debug)]
pub struct Metalink {}

impl Metalink {
    pub fn try_from_bytes(input: &[u8]) -> Result<Self, anyhow::Error> {
        let mut tree = XmlTree::try_from_bytes(input)?;

        if let Some(files) = tree.root_node.find_one("files") {
            for file in files.find_all("file") {
                if let Some(resources) = file.find_one("resources") {
                    resources.filter_children_in_place(|url| {
                        let address = url.characters.clone().unwrap_or_default();
                        address.starts_with("http")
                    });
                }
            }
        }

        println!("{tree:#?}");
        Ok(Metalink {})
    }
}