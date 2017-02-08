use std::fmt::Debug;
const HTML5_DOCTYPE: &'static str = "<!DOCTYPE html>";
/// Default `CharSet`
pub static mut CHARSET: &'static str = "UTF-8";

type Attr = (String, String);

trait AttrToString {
    fn attr_to_string(&self) -> (String, String);
}

impl<T: AsRef<str> + Debug> AttrToString for (T, T) {
    fn attr_to_string(&self) -> (String, String) {
        let (ref n, ref c) = *self;
        (n.as_ref().to_string(), c.as_ref().to_string())
    }
}
impl<'a, T: AsRef<str> + Debug + 'a> AttrToString for &'a (T, T) {
    fn attr_to_string(&self) -> (String, String) {
        let (ref n, ref c) = **self;
        (n.as_ref().to_string(), c.as_ref().to_string())
    }
}

/// Single tag, `<meta charset="UTF-8">` or `<p>`
#[derive(Debug)]
pub struct TagSingle {
    name: String,
    attrs: Vec<Attr>,
}

impl TagSingle {
    pub fn new<S: AsRef<str> + Debug>(name: S) -> Self {
        TagSingle {
            name: name.as_ref().to_string(),
            attrs: Vec::new(),
        }
    }
    /// `charset="UTF-8"`
    #[inline]
    pub fn add_attr<S: AsRef<str> + Debug>(mut self, name: S, content: S) -> Self {
        self.attrs.push((name, content).attr_to_string());
        self
    }
    pub fn add_attrs<'a, S: AsRef<str> + Debug + 'a, T: Iterator<Item = &'a (S, S)>>(mut self,
                                                                                     attrs: T)
                                                                                     -> Self {
        for &(ref name, ref content) in attrs {
            self = self.add_attr(name, content);
        }
        self
    }
}
// 内容(明文)
#[derive(Debug)]
struct Content {
    content: String,
}
impl Content {
    pub fn new<S: AsRef<str> + Debug>(content: S) -> Self {
        Content { content: content.as_ref().to_string() }
    }
}

/// Double tag, `<script type="text/javascript" src="/app.js"></script>`
#[derive(Debug)]
pub struct TagDouble {
    name: String,
    attrs: Vec<Attr>,
    nodes: Vec<Node>,
}
impl TagDouble {
    pub fn new<S: AsRef<str> + Debug>(name: S) -> Self {
        TagDouble {
            name: name.as_ref().to_string(),
            attrs: Vec::new(),
            nodes: Vec::new(),
        }
    }
    /// `type="text/javascript"` , `"src="/app.js"`
    #[inline]
    pub fn add_attr<S: AsRef<str> + Debug>(mut self, name: S, content: S) -> Self {
        self.attrs.push((name, content).attr_to_string());
        self
    }
    pub fn add_attrs<'a, S: AsRef<str> + Debug + 'a, T: Iterator<Item = &'a (S, S)>>(mut self,
                                                                                     attrs: T)
                                                                                     -> Self {
        for &(ref name, ref content) in attrs {
            self = self.add_attr(name, content);
        }
        self
    }
    pub fn add_tags(mut self, tags: TagSingle) -> Self {
        self.nodes.push(Node::ST(tags));
        self
    }
    pub fn add_tagd(mut self, tagd: TagDouble) -> Self {
        self.nodes.push(Node::DT(tagd));
        self
    }
    /// `127.0.0.1:58488` in `<span id="client"> 127.0.0.1:58488</span>`
    pub fn add_content<S: AsRef<str> + Debug>(mut self, content: S) -> Self {
        self.nodes.push(Node::CT(Content::new(content)));
        self
    }
}
#[derive(Debug)]
enum Node {
    ST(TagSingle),
    DT(TagDouble),
    CT(Content),
}

// const HTML5_META:  &'static str =r#"<meta charset=UTF-8">"#;
/// Manage the `CharSet` of the `Html`
#[derive(Debug)]
pub struct CharSet {}
impl CharSet {
    pub fn set(charset: &'static str) {
        unsafe {
            CHARSET = charset;
        }
    }
    pub fn get() -> &'static str {
        unsafe { CHARSET }
    }
}

/// `Html` struct
#[derive(Debug,Default)]
pub struct Html {
    doctype: String,
    nodes: Vec<Node>,
}

impl Html {
    pub fn new() -> Self {
        Html {
            doctype: HTML5_DOCTYPE.to_owned(),
            nodes: Vec::new(),
        }
    }
    pub fn add_tags(mut self, tags: TagSingle) -> Self {
        self.nodes.push(Node::ST(tags));
        self
    }
    pub fn add_tagd(mut self, tagd: TagDouble) -> Self {
        self.nodes.push(Node::DT(tagd));
        self
    }
    /// `127.0.0.1:58488` in `<html> 127.0.0.1:58488</html>`
    pub fn add_content<S: AsRef<str> + Debug>(mut self, content: S) -> Self {
        self.nodes.push(Node::CT(Content::new(content)));
        self
    }
    /// `TagDouble::new("head").add_tags(TagSingle::new("meta"))` being added `CHARSET`: `<head><meta charset="UTF-8"></head>`.
    pub fn head() -> TagDouble {
        TagDouble::new("head")
            .add_tags(TagSingle::new("meta").add_attr("charset", CharSet::get()))
            .add_tags(TagSingle::new("link").add_attrs(vec![("rel", "shortcut icon"),
                                                            ("type", "image/x-ico"),
                                                            ("href", "/favicon.ico")]
                .iter()))
            .add_tags(TagSingle::new("link").add_attrs(vec![("rel", "stylesheet"),
                                                            ("type", "text/css"),
                                                            ("href", "/style.css")]
                .iter()))
            .add_tagd(TagDouble::new("script")
                .add_attrs(vec![("type", "text/javascript"), ("src", "/app.js")].iter()))
    }
}

impl ToString for Html {
    fn to_string(&self) -> String {
        fn rec_nodes(mut html: String, nodes: &Vec<Node>) -> String {
            for node in nodes.as_slice().iter() {
                match *node {
                    Node::ST(ref st) => {
                        html = html + "<" + &st.name;
                        html = for_attrs(html, &st.attrs);
                        html += ">";
                    }
                    Node::DT(ref dt) => {
                        html = html + "<" + &dt.name;
                        html = for_attrs(html, &dt.attrs) + ">";
                        html = rec_nodes(html, &dt.nodes);
                        html = html + "</" + &dt.name + ">";
                    }
                    Node::CT(ref ct) => {
                        html = html + " " + &ct.content;
                    } 
                }
            }
            html
        }
        fn for_attrs(mut html: String, attrs: &Vec<Attr>) -> String {
            for &(ref name, ref content) in attrs.as_slice().iter() {
                html = html + " " + name + "=" + "\"" + content + "\"";
            }
            html
        }
        let mut html = self.doctype.clone();
        html += "<html>";
        html = rec_nodes(html, &self.nodes);
        html += "</html>";
        html
    }
}
