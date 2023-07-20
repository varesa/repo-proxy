use std::fmt::{Debug, Formatter, Write};
use xml::reader::XmlEvent;

#[derive(Clone)]
pub struct XmlNode {
    start_element: XmlEvent,
    characters: Option<String>,
    children: Vec<XmlNode>,
}

#[derive(Clone)]
pub struct XmlTree {
    start_document: XmlEvent,
    root_node: XmlNode,
}

static mut LIMIT: usize = 0;

impl XmlNode {
    pub fn try_from_events(events: &[XmlEvent]) -> Result<(Self, &[XmlEvent]), anyhow::Error> {
        unsafe { LIMIT += 1; assert!(LIMIT < 10000) }

        let mut characters = None;
        let mut children = Vec::new();

        assert!(!events.is_empty());
        if let XmlEvent::StartElement { .. } = events[0] {
            let mut remaining = &events[1..];
            let mut index = 0;

            loop {
                if remaining.is_empty() {
                    return Err(anyhow::Error::msg(format!("XmlNode: Unexpected end of stream. Last element: {:?}", &events.last().unwrap())));
                }

                match &remaining[index] {
                    XmlEvent::Characters(s) => {
                        characters = Some(s.clone());
                        index += 1;
                    },
                    XmlEvent::StartElement { .. } => {
                        let (node, remaining_) = XmlNode::try_from_events(&remaining[index..])?;
                        remaining = remaining_;
                        index = 0;
                        children.push(node);
                    },
                    XmlEvent::EndElement { .. } => {
                        assert!(!(characters.is_some() && !children.is_empty()));
                        return Ok((XmlNode {
                            start_element: events[0].clone(),
                            characters,
                            children
                        }, &remaining[index+1..]));
                    },
                    _ => unimplemented!("Event {:?} not implemented", &events[0])
                }
            }
        } else {
            return Err(anyhow::Error::msg(format!("XmlNode: Expected StartElement, got {:?}", events)));
        }
    }

    pub fn name(&self) -> String {
        if let XmlEvent::StartElement { name, ..} = &self.start_element {
            name.local_name.clone()
        } else {
            panic!("XmlNode: Expected StartElement, got {:?}", &self.start_element);
        }
    }

    pub fn find_one(&self, name: &str) -> Option<&XmlNode> {
        self.children.iter().find(|child| child.name() == name)
    }

    pub fn find_all(&self, name: &str) -> Vec<&XmlNode> {
        self.children.iter().filter(|child| child.name() == name).collect()
    }

    pub fn filter_children_in_place<P>(&mut self, predicate: P) where
        P: FnMut(&Self) -> bool,
    {
        let new_children: Vec<XmlNode> = self.children.clone().into_iter().filter(predicate).collect();
        self.children = new_children;
    }
}

impl Debug for XmlNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        if let XmlEvent::StartElement { name, .. } = &self.start_element {
            if let Some(s) = &self.characters {
                output.write_str(&format!(r#"  <{} ...>"{}"</{}>"#, name.local_name, s, name.local_name)).unwrap();
            } else if !&self.children.is_empty() {
                output.write_str(&format!("  <{} ...>\n", name.local_name)).unwrap();
                for child in &self.children {
                    let debug = format!("{:#?}", child);
                    for line in debug.lines() {
                        output.write_str(&format!("  {line}\n")).unwrap();
                    }
                }
                output.write_str(&format!("  </{}>", name.local_name)).unwrap();
            } else {
                output.write_str(&format!("  <{} .../>", name.local_name)).unwrap();
            }
            f.write_str(&output)
        } else {
            panic!("Invalid start element");
        }
    }
}

impl XmlTree {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        let mut parser = xml::reader::EventReader::new(bytes);
        let mut events = Vec::new();

        loop {
            let event = parser.next()?;
            match event {
                XmlEvent::Whitespace(_) => {},
                XmlEvent::EndDocument => {
                    break;
                },
                _ => { events.push(event) },
            }
        }

        assert!(!events.is_empty());
        let start_document = if let XmlEvent::StartDocument { .. } = events[0] {
            events[0].clone()
        } else {
            return Err(anyhow::Error::msg(format!("XmlTree: Expected StartDocument, got: {:?}", events)));
        };

        let (root_node, remaining) = XmlNode::try_from_events(&events[1..])?;
        assert!(remaining.is_empty());

        Ok(XmlTree {
            start_document,
            root_node,
        })
    }
}

impl Debug for XmlTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        out.write_str("XmlTree {\n").unwrap();
        out.write_str(&format!("    start_document: {:?}\n", &self.start_document)).unwrap();
        out.write_str("    root_node:\n").unwrap();
        let node = format!("{:#?}", &self.root_node);
        for line in node.lines() {
            out.write_str(&format!("      {line}\n")).unwrap();
        }
        out.write_str("}").unwrap();
        f.write_str(&out)
    }
}