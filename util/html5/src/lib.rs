#![allow(unknown_lints)]
#![allow(ptr_arg,len_without_is_empty)]
use std::fmt::{self, Debug};
use std::io;

const HTML5_DTD: &'static str = "<!DOCTYPE html>";
const HTML_TAG: &'static str = "html";
/// Default `CharSet`
static mut CHARSET: &'static str = "UTF-8";
type Attr = (String, String);

/// Manage the `CharSet` for the `HTML`: `<head><meta charset="UTF-8"></head>`
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

/// Content plaintext: `html5 - Rust` in `<title>html5 - Rust</title>`
///
/// You can also insert the source of HTML directly: `</p>` , `<title>html5 - Rust</title>` , etc.
#[derive(Debug)]
pub struct Content {
    content: String,
}
impl Content {
    pub fn new<S>(content: S) -> Self
        where S: Into<String> + Debug
    {
        Content { content: content.into() }
    }
}

/// Single tag: `<meta charset="UTF-8">` or `<p>`
#[derive(Debug)]
pub struct TagSingle {
    name: String,
    attrs: Vec<Attr>,
}

impl TagSingle {
    pub fn new<S>(name: S) -> Self
        where S: Into<String> + Debug
    {
        TagSingle {
            name: name.into(),
            attrs: Vec::new(),
        }
    }
    /// `charset="UTF-8"`
    #[inline]
    pub fn add_attr<S0, S1>(mut self, name: S0, content: S1) -> Self
        where S0: Into<String> + Debug,
              S1: Into<String> + Debug
    {
        self.attrs.push((name.into(), content.into()));
        self
    }
    pub fn add_attrs<'a, S0, S1, T>(mut self, attrs: T) -> Self
        where S0: ToString + Debug + 'a,
              S1: ToString + Debug + 'a,
              T: Iterator<Item = &'a (S0, S1)>
    {
        for &(ref name, ref content) in attrs {
            self = self.add_attr((*name).to_string(), (*content).to_string());
        }
        self
    }
}

/// Double tag: `<script type="text/javascript" src="/app.js"></script>`
//暂时不要把TagDouble里直接套TagSingle,因为Debug的可读性,以及需要改ToString
#[derive(Debug)]
pub struct TagDouble {
    name: String,
    attrs: Vec<Attr>,
    nodes: Vec<Node>,
}
impl TagDouble {
    pub fn new<S>(name: S) -> Self
        where S: Into<String> + Debug
    {
        TagDouble {
            name: name.into(),
            attrs: Vec::new(),
            nodes: Vec::new(),
        }
    }
    /// `type="text/javascript"` , `"src="/app.js"`
    #[inline]
    pub fn add_attr<S0, S1>(mut self, name: S0, content: S1) -> Self
        where S0: Into<String> + Debug,
              S1: Into<String> + Debug
    {
        self.attrs.push((name.into(), content.into()));
        self
    }
    pub fn add_attrs<'a, S0, S1, T>(mut self, attrs: T) -> Self
        where S0: ToString + Debug + 'a,
              S1: ToString + Debug + 'a,
              T: Iterator<Item = &'a (S0, S1)>
    {
        for &(ref name, ref content) in attrs {
            self = self.add_attr((*name).to_string(), (*content).to_string());
        }
        self
    }
    pub fn push<T: IntoNode>(mut self, tag: T) -> Self {
        self.nodes.push(tag.into_node());
        self
    }
}

/// `Content,TagSingle,TagDouble`
#[derive(Debug)]
pub enum Node {
    TS(TagSingle),
    TD(TagDouble),
    CT(Content),
}

///`impl IntoNode for S` <=> `Content::new(self).into_node()`
pub trait IntoNode {
    fn into_node(self) -> Node;
}
impl IntoNode for TagSingle {
    fn into_node(self) -> Node {
        Node::TS(self)
    }
}
impl IntoNode for TagDouble {
    fn into_node(self) -> Node {
        Node::TD(self)
    }
}
impl IntoNode for Content {
    fn into_node(self) -> Node {
        Node::CT(self)
    }
}

impl<S: Into<String> + Debug> IntoNode for S {
    fn into_node(self) -> Node
        where S: Into<String> + Debug
    {
        Content::new(self).into_node()
    }
}

/// Struct `HTML`
#[derive(Debug,Default)]
pub struct HTML {
    dtd: String,
    nodes: Vec<Node>,
}

impl HTML {
    /// With HTML5_DTD: `<!DOCTYPE html>`
    pub fn new() -> Self {
        Self::with_dtd(HTML5_DTD)
    }
    pub fn with_dtd<S: Into<String> + Debug>(dtd: S) -> Self {
        Self {
            dtd: dtd.into(),
            nodes: Vec::new(),
        }
    }
    pub fn push<T: IntoNode>(mut self, tag: T) -> Self {
        self.nodes.push(tag.into_node());
        self
    }
    /// `TagDouble::new("head")` being added `CHARSET` and icon:
    ///
    ///`<head><meta charset="UTF-8"><link rel="shortcut icon" type="image/x-ico" href="/favicon.ico"></head>`
    pub fn head() -> TagDouble {
        TagDouble::new("head")
            .push(TagSingle::new("meta").add_attr("charset", CharSet::get()))
            .push(TagSingle::new("link").add_attrs(vec![("rel", "shortcut icon"), ("type", "image/x-ico"), ("href", "/favicon.ico")].iter()))
    }
    pub fn head_with_charset<S: Into<String> + Debug>(charset: S) -> TagDouble {
        TagDouble::new("head")
            .push(TagSingle::new("meta").add_attr("charset", charset.into()))
            .push(TagSingle::new("link").add_attrs(vec![("rel", "shortcut icon"), ("type", "image/x-ico"), ("href", "/favicon.ico")].iter()))
    }
    #[doc(hidden)]
    pub fn len(&self) -> usize {
        fn rec_nodes(mut count: usize, nodes: &Vec<Node>) -> usize {
            for node in nodes.as_slice().iter() {
                match *node {
                    Node::TS(ref ts) => {
                        count += ts.name.len() + "<>".len();
                        count = for_attrs(count, &ts.attrs);
                    }
                    Node::TD(ref td) => {
                        count += td.name.len() + "<>".len();
                        count = for_attrs(count, &td.attrs);
                        count = rec_nodes(count, &td.nodes);
                        count += td.name.len() + "</>".len();
                    }
                    Node::CT(ref ct) => {
                        count += ct.content.len();
                    }
                }
            }
            fn for_attrs(mut count: usize, attrs: &Vec<Attr>) -> usize {
                for &(ref name, ref content) in attrs.as_slice().iter() {
                    count += r#" ="""#.len() + name.len() + content.len();
                }
                count
            }
            count
        }
        let mut count = 0usize;
        count += self.dtd.len();
        count += HTML_TAG.len() * 2 + "<>".len() + "</>".len();
        rec_nodes(count, &self.nodes)
    }
    pub fn write(&self, mut w: &mut io::Write) -> io::Result<usize> {
        fn rec_nodes(mut w: &mut io::Write, mut count: usize, nodes: &Vec<Node>) -> io::Result<usize> {
            for node in nodes.as_slice().iter() {
                match *node {
                    Node::TS(ref ts) => {
                        count += w.write(&[b'<'])?;
                        count += w.write(ts.name.as_bytes())?;
                        count = for_attrs(w, count, &ts.attrs)?;
                        count += w.write(&[b'>'])?;
                    }
                    Node::TD(ref td) => {
                        count += w.write(&[b'<'])?;
                        count += w.write(td.name.as_bytes())?;
                        count = for_attrs(w, count, &td.attrs)?;
                        count += w.write(&[b'>'])?;
                        count = rec_nodes(w, count, &td.nodes)?;
                        count += w.write(b"</")?;
                        count += w.write(td.name.as_bytes())?;
                        count += w.write(&[b'>'])?;
                    }
                    Node::CT(ref ct) => {
                        count += w.write(ct.content.as_bytes())?;
                    }
                }
            }
            fn for_attrs(mut w: &mut io::Write, mut count: usize, attrs: &Vec<Attr>) -> io::Result<usize> {
                for &(ref name, ref content) in attrs.as_slice().iter() {
                    count += w.write(&[b' '])?;
                    count += w.write(name.as_bytes())?;
                    count += w.write(&[b'='])?;
                    count += w.write(&[b'"'])?;
                    count += w.write(content.as_bytes())?;
                    count += w.write(&[b'"'])?;
                }
                Ok(count)
            }
            Ok(count)
        }
        let mut count = 0usize; // Could't use &mut,because of `Copy`
        count += w.write(self.dtd.as_bytes())?;
        count += w.write(format!("<{}>", HTML_TAG).as_bytes())?;
        count = rec_nodes(w, count, &self.nodes)?;
        count += w.write(format!("</{}>", HTML_TAG).as_bytes())?;
        Ok(count)
    }
    // #[inline]
    // pub fn write_all(&self, mut w: &mut io::Write) -> io::Result<()> {
    //     let _ = self.write(w)?;
    //     Ok(())
    // }
    pub fn write_all(&self, mut w: &mut io::Write) -> io::Result<()> {
        fn rec_nodes(mut w: &mut io::Write, nodes: &Vec<Node>) -> io::Result<()> {
            for node in nodes.as_slice().iter() {
                match *node {
                    Node::TS(ref ts) => {
                        w.write_fmt(format_args!("<{}", ts.name))?;
                        for_attrs(w, &ts.attrs)?;
                        w.write_all(&[b'>'])?;
                    }
                    Node::TD(ref td) => {
                        w.write_fmt(format_args!("<{}", td.name))?;
                        for_attrs(w, &td.attrs)?;
                        w.write_all(&[b'>'])?;
                        rec_nodes(w, &td.nodes)?;
                        w.write_fmt(format_args!("</{}>", td.name))?;
                    }
                    Node::CT(ref ct) => {
                        w.write_all(ct.content.as_bytes())?;
                    }
                }
            }
            fn for_attrs(mut w: &mut io::Write, attrs: &Vec<Attr>) -> io::Result<()> {
                for &(ref name, ref content) in attrs.as_slice().iter() {
                    w.write_fmt(format_args!(r#" {}="{}""#, name, content))?;
                }
                Ok(())
            }
            Ok(())
        }
        w.write_all(self.dtd.as_bytes())?;
        w.write_fmt(format_args!("<{}>", HTML_TAG))?;
        rec_nodes(w, &self.nodes)?;
        w.write_fmt(format_args!("</{}>", HTML_TAG))?;
        Ok(())
    }
    #[inline]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut html: Vec<u8> = Vec::new();
        self.write_all(&mut html).unwrap();
        html
    }
    pub fn write_fmt(&self, mut w: &mut fmt::Write) -> Result<(), fmt::Error> {
        fn rec_nodes(mut w: &mut fmt::Write, nodes: &Vec<Node>) -> Result<(), fmt::Error> {
            for node in nodes.as_slice().iter() {
                match *node {
                    Node::TS(ref ts) => {
                        w.write_fmt(format_args!("<{}", &ts.name))?;
                        for_attrs(w, &ts.attrs)?;
                        w.write_char('>')?;
                    }
                    Node::TD(ref td) => {
                        w.write_fmt(format_args!("<{}", &td.name))?;
                        for_attrs(w, &td.attrs)?;
                        w.write_char('>')?;
                        rec_nodes(w, &td.nodes)?;
                        w.write_fmt(format_args!("</{}>", &td.name))?;
                    }
                    Node::CT(ref ct) => {
                        w.write_str(&ct.content)?;
                    }
                }
            }
            fn for_attrs(mut w: &mut fmt::Write, attrs: &Vec<Attr>) -> Result<(), fmt::Error> {
                for &(ref name, ref content) in attrs.as_slice().iter() {
                    w.write_fmt(format_args!(r#" {}="{}""#, &name, &content))?;
                }
                Ok(())
            }
            Ok(())
        }
        w.write_str(&self.dtd)?;
        w.write_str(format!("<{}>", HTML_TAG).as_str())?;
        rec_nodes(w, &self.nodes)?;
        w.write_str(format!("</{}>", HTML_TAG).as_str())?;
        Ok(())
    }
}
impl ToString for HTML {
    #[inline]
    fn to_string(&self) -> String {
        let mut html = String::new();
        self.write_fmt(&mut html).unwrap();
        html
    }
}
