use std::fmt::{Debug, Formatter, Write};
use xml::reader::XmlEvent;

#[derive(Clone)]
pub struct XmlNode {
    pub start_element: XmlEvent,
    pub characters: Option<String>,
    pub children: Vec<XmlNode>,
}

#[derive(Clone)]
pub struct XmlTree {
    pub start_document: XmlEvent,
    pub root_node: XmlNode,
}

static mut LIMIT: usize = 0;

impl XmlNode {
    /// Recursively create XmlNodes from a linear list of XmlEvents
    pub fn try_from_events(events: &[XmlEvent]) -> Result<(Self, &[XmlEvent]), anyhow::Error> {
        assert!(!events.is_empty(), "XmlNode::try_from_events called without any events");

        // Add a safeguard against infinite looping and skyrocketing memory usage
        unsafe { LIMIT += 1; assert!(LIMIT < 10000) }

        // A node can have either text content, child nodes, or neither
        let mut characters = None;
        let mut children = Vec::new();

        // The first event should be StartElement. If not, something went wrong and we error out
        if let XmlEvent::StartElement { .. } = events[0] {

            // Remaining tokens, including but not limited to the child elements of this node
            // and the current index to them
            let mut remaining = &events[1..];
            let mut index = 0;

            loop {
                if remaining.is_empty() {
                    return Err(anyhow::Error::msg(format!("XmlNode: Unexpected end of stream. Last element: {:?}", &events.last().unwrap())));
                }

                match &remaining[index] {
                    // Node with a text content. Expect the next event to be an EndElement
                    XmlEvent::Characters(s) => {
                        characters = Some(s.clone());
                        index += 1;
                    },
                    // A node with child nodes
                    XmlEvent::StartElement { .. } => {
                        // Recursively start processing a new child node
                        let (node, remaining_) = XmlNode::try_from_events(&remaining[index..])?;
                        children.push(node);

                        // Fast-forward the list of remaining tokens by the amount consumed
                        // by the child node above
                        remaining = remaining_;
                        index = 0;
                    },
                    XmlEvent::EndElement { .. } => {
                        // Verify that we didn't end up with a node with both inner text
                        // and children. Go back one level in the recursion
                        assert!(!(characters.is_some() && !children.is_empty()));
                        return Ok((XmlNode {
                            start_element: events[0].clone(),
                            characters,
                            children
                        }, &remaining[index+1..]));
                    },
                    // Some things like comments are not handled
                    _ => unimplemented!("Event {:?} not implemented", &events[0])
                }
            }
        } else {
            return Err(anyhow::Error::msg(format!("XmlNode: Expected StartElement, got {:?}", events)));
        }
    }

    /// A shortcut to cast the StartElement and avoid an if-let
    pub fn name(&self) -> String {
        if let XmlEvent::StartElement { name, ..} = &self.start_element {
            name.local_name.clone()
        } else {
            panic!("XmlNode: Expected StartElement, got {:?}", &self.start_element);
        }
    }

    /// Return a reference to one (or none) child given the tag name
    pub fn find_one(&mut self, name: &str) -> Option<&mut XmlNode> {
        self.children.iter_mut().find(|child| child.name() == name)
    }

    /// Return a list of references to all children given the tag name
    pub fn find_all(&mut self, name: &str) -> Vec<&mut XmlNode> {
        self.children.iter_mut().filter(|child| child.name() == name).collect()
    }

    /// Remove any children that do not meet the predicate
    pub fn filter_children_in_place<P>(&mut self, predicate: P) where
        P: FnMut(&Self) -> bool,
    {
        let new_children: Vec<XmlNode> = self.children.clone().into_iter().filter(predicate).collect();
        self.children = new_children;
    }
}

/// Custom Debug-format that resembles pretty-printed XML and is denser than the default #? format
impl Debug for XmlNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        if let Some(s) = &self.characters {
            // Format 1: Tag with text content
            output.write_str(&format!(r#"  <{} ...>"{}"</{}>"#, self.name(), s, self.name())).unwrap();

        } else if !&self.children.is_empty() {
            // Format 2: Tag with children
            output.write_str(&format!("  <{} ...>\n", self.name())).unwrap();
            for child in &self.children {
                let debug = format!("{:#?}", child);
                for line in debug.lines() {
                    output.write_str(&format!("  {line}\n")).unwrap();
                }
            }
            output.write_str(&format!("  </{}>", self.name())).unwrap();

        } else {
            // Format 3: Tag with neither text nor children
            output.write_str(&format!("  <{} .../>", self.name())).unwrap();
        }
        f.write_str(&output)
    }
}

impl XmlTree {
    /// Convert a list of bytes into a tree
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        // Create a parser to convert the string/byte format XML to a linear stream of tokens
        let mut parser = xml::reader::EventReader::new(bytes);
        let mut events = Vec::new();

        // Remove whitespace to simplify tree construction
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

        assert!(!events.is_empty(), "Empty document or error parsing");

        // The document should start with an <?xml> tag
        let start_document = if let XmlEvent::StartDocument { .. } = events[0] {
            events[0].clone()
        } else {
            return Err(anyhow::Error::msg(format!("XmlTree: Expected StartDocument, got: {:?}", events)));
        };

        // Assume single root node following the start tag and convert to a tree
        let (root_node, remaining) = XmlNode::try_from_events(&events[1..])?;
        assert!(remaining.is_empty(), "tokens left over after constructing tree. Multiple root nodes?");

        Ok(XmlTree {
            start_document,
            root_node,
        })
    }
}

/// Slightly modified #? format that produces better matched whitespace
/// with the multi-line root_node
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