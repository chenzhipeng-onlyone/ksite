use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The Root of AST.
struct Package<'a> {
    name: &'a [u8],
    syntax: &'a [u8],
    imports: Vec<&'a [u8]>,
    entries: Vec<Entry<'a>>,
}

impl<'a> Package<'a> {
    fn new(s: &'a TokenStream) -> Self {
        let mut ret = Self {
            name: b"",
            syntax: b"",
            imports: Vec::new(),
            entries: Vec::new(),
        };
        assert!(s.next().1 == b"syntax");
        s.next(); // '='
        s.next(); // '"'
        ret.syntax = s.next().1;
        s.next(); // '"'
        s.next(); // ';'
        assert!(s.next().1 == b"package");
        ret.name = s.next().1;
        s.next(); // ';'
        while let Some(token) = s.peek() {
            match (&token.0, token.1) {
                (TokenKind::Word, b"import") => {
                    s.next();
                    s.next(); // '"'
                    ret.imports.push(s.next().1);
                    s.next(); // '"'
                    s.next(); // ';'
                }
                (TokenKind::Word, b"enum") => {
                    ret.entries.push(Entry::Enum(Enum::new(s)));
                }
                (TokenKind::Word, b"message") => {
                    ret.entries.push(Entry::Message(Message::new(s)));
                }
                _ => unreachable!(),
            };
        }
        ret
    }
}

enum Entry<'a> {
    Message(Message<'a>),
    MessageField(MessageField<'a>),
    Enum(Enum<'a>),
    Oneof(Oneof<'a>),
}

struct Message<'a> {
    name: &'a [u8],
    entries: Vec<Entry<'a>>,
}

impl<'a> Message<'a> {
    fn new(s: &'a TokenStream) -> Self {
        let mut ret = Self {
            name: b"",
            entries: Vec::new(),
        };
        assert!(s.next().1 == b"message");
        ret.name = s.next().1;
        s.next(); // '{'
        while let Some(token) = s.peek() {
            match (&token.0, token.1) {
                (TokenKind::Symbol, b"}") => {
                    s.next();
                    if let Some((_, b";")) = s.peek() {
                        s.next();
                    }
                    break;
                }
                (TokenKind::Word, b"message") => {
                    ret.entries.push(Entry::Message(Message::new(s)));
                }
                (TokenKind::Word, b"oneof") => {
                    ret.entries.push(Entry::Oneof(Oneof::new(s)));
                }
                (TokenKind::Word, b"enum") => {
                    ret.entries.push(Entry::Enum(Enum::new(s)));
                }
                _ => {
                    ret.entries.push(Entry::MessageField(MessageField::new(s)));
                }
            };
        }
        ret
    }
}

struct MessageField<'a> {
    name: &'a [u8],
    data_type: &'a [u8],
    tag: &'a [u8],
    optional: bool,
    repeated: bool,
}

impl<'a> MessageField<'a> {
    fn new(s: &'a TokenStream) -> Self {
        let mut ret = Self {
            name: b"",
            data_type: b"",
            tag: b"",
            optional: false,
            repeated: false,
        };
        while let Some(token) = s.peek() {
            match (&token.0, token.1) {
                (TokenKind::Word, b"optional") => {
                    s.next();
                    ret.optional = true;
                }
                (TokenKind::Word, b"repeated") => {
                    s.next();
                    ret.repeated = true;
                }
                (TokenKind::Word, _) if ret.data_type.is_empty() => {
                    ret.data_type = s.next().1;
                }
                (TokenKind::Word, _) if ret.name.is_empty() => {
                    ret.name = s.next().1;
                }
                (TokenKind::Symbol, b"=") => {
                    s.next();
                    ret.tag = s.next().1;
                    s.next(); // ';'
                    break;
                }
                _ => unreachable!(),
            };
        }
        ret
    }
}

struct Enum<'a> {
    name: &'a [u8],
    fields: Vec<EnumField<'a>>,
}

struct EnumField<'a> {
    name: &'a [u8],
    tag: &'a [u8],
}

impl<'a> Enum<'a> {
    fn new(s: &'a TokenStream) -> Self {
        let mut ret = Self {
            name: b"",
            fields: Vec::new(),
        };
        assert!(s.next().1 == b"enum");
        ret.name = s.next().1;
        s.next(); // '{'
        while let Some(token) = s.peek() {
            match (&token.0, token.1) {
                (TokenKind::Symbol, b"}") => {
                    s.next();
                    if let Some((_, b";")) = s.peek() {
                        s.next();
                    }
                    break;
                }
                (TokenKind::Word, _) => {
                    let name = s.next().1;
                    s.next(); // '='
                    let tag = s.next().1;
                    s.next(); // ';'
                    ret.fields.push(EnumField { name, tag });
                }
                _ => unreachable!(),
            };
        }
        ret
    }
}

struct Oneof<'a> {
    name: &'a [u8],
    fields: Vec<OneofField<'a>>,
}

struct OneofField<'a> {
    name: &'a [u8],
    data_type: &'a [u8],
    tag: &'a [u8],
}

impl<'a> Oneof<'a> {
    fn new(s: &'a TokenStream) -> Self {
        let mut ret = Self {
            name: b"",
            fields: Vec::new(),
        };
        assert!(s.next().1 == b"oneof");
        ret.name = s.next().1;
        s.next(); // '{'
        while let Some(token) = s.peek() {
            match (&token.0, token.1) {
                (TokenKind::Symbol, b"}") => {
                    s.next();
                    if let Some((_, b";")) = s.peek() {
                        s.next();
                    }
                    break;
                }
                _ => {
                    let data_type = s.next().1;
                    let name = s.next().1;
                    s.next(); // '='
                    let tag = s.next().1;
                    s.next(); // ';'
                    ret.fields.push(OneofField {
                        name,
                        data_type,
                        tag,
                    });
                }
            };
        }
        ret
    }
}

enum TokenKind {
    Symbol,
    Number,
    Word,
    End,
}

type Token<'a> = (TokenKind, &'a [u8]);

struct TokenStream<'a> {
    tokens: Vec<Token<'a>>,
    idx: Cell<usize>,
}

impl<'a> TokenStream<'a> {
    fn peek(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.idx.get())
    }

    fn next(&self) -> &Token<'a> {
        let idx = self.idx.get();
        self.idx.set(idx + 1);
        self.tokens.get(idx).unwrap()
    }

    /// Create `TokenStream` from proto file data.
    fn new(mut s: &'a [u8]) -> Self {
        let mut tokens = Vec::new();
        loop {
            let token = next_token(&mut s);
            if let TokenKind::End = token.0 {
                break;
            }
            tokens.push(token);
        }
        Self {
            tokens,
            idx: Cell::new(0),
        }
    }
}

fn next_token<'a>(s: &mut &'a [u8]) -> Token<'a> {
    let mut kind = TokenKind::End;
    let mut begun = false;
    let mut range = (0, 0);
    let found = |from, c| from + s[from..].iter().position(|&v| v == c).unwrap();
    let is_symbol = |c| matches!(c, b'{' | b'}' | b'=' | b';' | b'"');
    while let Some(&v) = s.get(range.1) {
        match begun {
            false if v == b'/' => {
                // comments
                range.1 += 1;
                match s[range.1] {
                    b'/' => range.1 = found(range.1 + 1, b'\n'),
                    b'*' => loop {
                        range.1 = found(range.1 + 1, b'*');
                        if s[range.1 + 1] == b'/' {
                            range.1 += 2;
                            break;
                        }
                    },
                    _ => unreachable!(),
                }
            }
            false if v.is_ascii_whitespace() => {
                range.1 += 1;
            }
            false => {
                kind = match v {
                    _ if is_symbol(v) => TokenKind::Symbol,
                    _ if v.is_ascii_digit() => TokenKind::Number,
                    _ => TokenKind::Word,
                };
                range.0 = range.1;
                begun = true;
            }
            true => match kind {
                TokenKind::Symbol => {
                    range.1 += 1;
                    break;
                }
                TokenKind::Number if v.is_ascii_digit() => {
                    range.1 += 1;
                }
                TokenKind::Word if !is_symbol(v) && !v.is_ascii_whitespace() => {
                    range.1 += 1;
                }
                _ => break,
            },
        }
    }
    let ret = (kind, &s[range.0..range.1]);
    *s = &s[range.1..];
    ret
}

/// Split an identifier into sections for case transforms later.
fn to_any_case(s: &[u8]) -> Vec<&[u8]> {
    let mut parts = Vec::<&[u8]>::new();
    let mut range = (0, 0);
    'label: while let Some(&f) = s.get(range.1) {
        range.1 += 1;
        let (mut r_u, mut r_d) = (f.is_ascii_uppercase(), f.is_ascii_uppercase());
        while let Some(&c) = s.get(range.1) {
            let (c_u, c_d) = (c.is_ascii_uppercase(), c.is_ascii_digit());
            match (r_u, r_d, c_u, c_d) {
                _ if c == b'_' => {
                    parts.push(&s[range.0..range.1]);
                    range.1 += 1; // skip current char
                    range.0 = range.1;
                    continue 'label;
                }
                (_, _, true, _) if range.1 + 1 < s.len() && s[range.1 + 1].is_ascii_lowercase() => {
                    break;
                }
                (_, _, _, true) | (true, _, true, false) | (_, _, false, false) => {
                    range.1 += 1;
                }
                (false, _, true, false) => break,
                // v => panic!("illegal state {:?}", v),
            }
            if !c_d {
                r_u = c_u;
            }
            r_d = c_d;
        }
        parts.push(&s[range.0..range.1]);
        range.0 = range.1;
    }
    parts
}

#[cfg(target_feature = "tests")]
fn test_to_any_case() {
    fn test_once(i: &str) {
        // to_any_case(i.as_bytes())
        //     .into_iter()
        //     .map(|v| String::from_utf8(v.to_vec()).unwrap())
        //     .for_each(|v| {
        //         println!("{v}");
        //     });

        use heck::{ToSnakeCase, ToUpperCamelCase};

        let expect = i.to_upper_camel_case();
        let mut o = Vec::new();
        push_big_camel(i.as_bytes(), &mut o);
        let ans = String::from_utf8(o).unwrap();
        // dbg!(&ans);
        assert_eq!(expect, ans, "to_big_camel wrong");

        let expect = i.to_snake_case();
        let mut o = Vec::new();
        push_snake(i.as_bytes(), &mut o);
        let ans = String::from_utf8(o).unwrap();
        assert_eq!(expect, ans, "to_snake wrong");
    }
    test_once("ABC4Defg");
    test_once("abc4defg");
    test_once("ABC4DEFG");
    test_once("abC4dEfg");
    test_once("abC4d_efg");
    test_once("ab3efg");
    test_once("ab3Efg");
    test_once("abcDA3Eg");
    test_once("abcDA3EFg");
    test_once("abcDEFg");
    test_once("c2CReadReport");
}

#[rustfmt::skip]
#[inline]
fn is_rust_key_word(i: &[u8]) -> bool { matches!(i,
// https://doc.rust-lang.org/std/index.html#keywords
b"Self"|b"as"|b"async"|b"await"|b"break"|b"const"|b"continue"|b"crate"|b"dyn"|
b"else"|b"enum"|b"extern"|b"false"|b"fn"|b"for"|b"if"|b"impl"|b"in"|b"let"|b"loop"|
b"match"|b"mod"|b"move"|b"mut"|b"pub"|b"ref"|b"return"|b"self"|b"static"|b"struct"|
b"super"|b"trait"|b"true"|b"type"|b"union"|b"unsafe"|b"use"|b"where"|b"while")
}

/// Push the identifier as big camel case.
fn push_big_camel(i: &[u8], o: &mut Vec<u8>) {
    let parts = to_any_case(i);
    let parts_len = parts.len();
    if parts_len == 1 && is_rust_key_word(parts[0]) {
        o.extend(b"r#");
    }
    for part in parts {
        o.push(part[0].to_ascii_uppercase());
        for c in &part[1..] {
            o.push(c.to_ascii_lowercase());
        }
    }
}

/// Push the identifier as snake case.
fn push_snake(i: &[u8], o: &mut Vec<u8>) {
    let parts = to_any_case(i);
    let parts_len = parts.len();
    if parts_len == 1 && is_rust_key_word(parts[0]) {
        o.extend(b"r#");
    }
    for part in parts {
        for c in part {
            o.push(c.to_ascii_lowercase());
        }
        o.push(b'_');
    }
    o.pop();
}

fn push_indent(n: i32, o: &mut Vec<u8>) {
    for _ in 0..n {
        o.extend(b"    ");
    }
}

fn push_mod_path(depth_diff: i32, cur_mod: &[u8], o: &mut Vec<u8>) {
    match depth_diff {
        0 => {
            push_snake(cur_mod, o);
            o.extend(b"::");
        }
        1 => {}
        2..=i32::MAX => {
            for _ in 0..(depth_diff - 1) {
                o.extend(b"super::");
            }
        }
        _ => unreachable!(),
    }
}

fn to_rust_type(i: &[u8]) -> &'static [u8] {
    match i {
        b"bool" => b"bool",
        b"float" => b"f32",
        b"double" => b"f64",
        b"int32" | b"sint32" | b"sfixed32" => b"i32",
        b"int64" | b"sint64" | b"sfixed64" => b"i64",
        b"uint32" | b"fixed32" => b"u32",
        b"uint64" | b"fixed64" => b"u64",
        b"string" => b"::prost::alloc::string::String",
        b"bytes" => b"::prost::alloc::vec::Vec<u8>",
        _ => b"custom",
    }
}

/// Translate AST to Rust source code.
fn translate(package: &Package) -> Vec<u8> {
    // https://developers.google.com/protocol-buffers/docs/proto
    // https://developers.google.com/protocol-buffers/docs/proto3

    type Context<'a> = HashMap<&'a [u8], (&'static [u8], i32)>; // <name, (type, depth)>
    let mut ctx = Context::new(); // names context
    let mut o = Vec::<u8>::new();

    fn handle_message(message: &Message, pbv: &[u8], ctx: &Context, depth: i32, o: &mut Vec<u8>) {
        let mut ctx = ctx.clone(); // sub context
        let mut has_nested = false;
        push_indent(depth, o);
        o.extend(b"#[derive(Clone, PartialEq, ::prost::Message)]\n");
        push_indent(depth, o);
        o.extend(b"pub struct ");
        push_big_camel(message.name, o);
        o.extend(b" {\n");
        for entry in &message.entries {
            match entry {
                Entry::Enum(inner) => {
                    has_nested = true;
                    ctx.insert(inner.name, (b"enum", depth));
                }
                Entry::Message(inner) => {
                    has_nested = true;
                    ctx.insert(inner.name, (b"message", depth));
                }
                Entry::Oneof(_) => {
                    has_nested = true;
                    // oneof is anonymous
                }
                Entry::MessageField(_) => {}
            }
        }
        for entry in &message.entries {
            match entry {
                Entry::MessageField(field) => {
                    // # From proto doc
                    // For string, bytes, and message fields, optional is compatible with
                    // repeated. Given serialized data of a repeated field as input, clients that
                    // expect this field to be optional will take the last input value if it's a
                    // primitive type field or merge all input elements if it's a message type
                    // field. Note that this is not generally safe for numeric types, including
                    // bools and enums. Repeated fields of numeric types can be serialized in the
                    // packed format, which will not be parsed correctly when an optional field
                    // is expected.
                    let rust_type = to_rust_type(field.data_type);
                    let is_in_ctx = ctx.get(field.data_type).is_some();
                    let is_enum = is_in_ctx && ctx[field.data_type].0 == b"enum";
                    let is_optional = {
                        let is_prime = is_enum || rust_type != b"custom";
                        if field.optional && field.repeated {
                            assert!(matches!(field.data_type, b"string" | b"bytes") || !is_prime);
                        }
                        field.optional || (!is_prime && !field.repeated)
                    };

                    // attr macros
                    push_indent(depth + 1, o);
                    o.extend(b"#[prost(");
                    if is_enum {
                        o.extend(b"enumeration=\"");
                        push_mod_path(depth - ctx[field.data_type].1, message.name, o);
                        push_big_camel(field.data_type, o);
                        o.extend(b"\", ");
                    } else if rust_type == b"custom" {
                        o.extend(b"message, ");
                    } else if field.data_type == b"bytes" {
                        o.extend(b"bytes=\"vec\", ");
                    } else {
                        o.extend(field.data_type);
                        o.extend(b", ");
                    }
                    if is_optional {
                        o.extend(b"optional, ");
                    }
                    if field.repeated {
                        o.extend(b"repeated, ");
                    }
                    // # From proto doc
                    // The packed option can be enabled for repeated primitive fields to
                    // enable a more efficient representation on the wire. Rather than
                    // repeatedly writing the tag and type for each element, the entire array
                    // is encoded as a single length-delimited blob. In proto3, only explicit
                    // setting it to false will avoid using packed encoding.
                    if pbv == b"proto2"
                        && field.repeated
                        && rust_type != b"custom"
                        && field.data_type != b"bytes"
                        && field.data_type != b"string"
                    {
                        o.extend(b"packed=\"false\", ");
                    }
                    o.extend(b"tag=\"");
                    o.extend(field.tag);
                    o.extend(b"\", ");
                    if *o.last().unwrap() == b' ' {
                        o.pop();
                        o.pop();
                    }
                    o.extend(b")]\n");

                    // value
                    push_indent(depth + 1, o);
                    o.extend(b"pub ");
                    push_snake(field.name, o);
                    o.extend(b": ");
                    let mut field_depth = 0;
                    // don't use `field.optional`, that only rely on whether `optional` keyword
                    // appeared in source file and AST. see `is_optional`'s define above.
                    if is_optional {
                        o.extend(b"::core::option::Option<");
                        field_depth += 1;
                    }
                    if field.repeated {
                        o.extend(b"::prost::alloc::vec::Vec<");
                        field_depth += 1;
                    }
                    if is_enum {
                        o.extend(b"i32");
                    } else if is_in_ctx {
                        push_mod_path(depth - ctx[field.data_type].1, message.name, o);
                        push_big_camel(field.data_type, o);
                    } else if rust_type == b"custom" {
                        push_big_camel(field.data_type, o);
                    } else {
                        o.extend(rust_type);
                    }
                    for _ in 0..field_depth {
                        o.extend(b">");
                    }
                    o.extend(b",\n");
                }
                Entry::Oneof(oneof) => {
                    // attr macros
                    push_indent(depth + 1, o);
                    o.extend(b"#[prost(oneof=\"");
                    push_snake(message.name, o);
                    o.extend(b"::");
                    push_big_camel(oneof.name, o);
                    o.extend(b"\", tags=\"");
                    for field in &oneof.fields {
                        o.extend(field.tag);
                        o.extend(b", ");
                    }
                    if *o.last().unwrap() == b' ' {
                        o.pop();
                        o.pop();
                    }
                    o.extend(b"\")]\n");

                    // value
                    push_indent(depth + 1, o);
                    o.extend(b"pub ");
                    push_snake(oneof.name, o);
                    o.extend(b": ::core::option::Option<");
                    push_snake(message.name, o);
                    o.extend(b"::");
                    push_big_camel(oneof.name, o);
                    o.extend(b">,\n");
                }
                Entry::Message(_) => {}
                Entry::Enum(_) => {}
            }
        }
        push_indent(depth, o);
        o.extend(b"}\n");
        if !has_nested {
            return;
        }
        push_indent(depth, o);
        o.extend(b"/// Nested message and enum types in `");
        o.extend(message.name);
        o.extend(b"`.\n");
        push_indent(depth, o);
        o.extend(b"pub mod ");
        push_snake(message.name, o);
        o.extend(b" {\n");
        for entry in &message.entries {
            match entry {
                Entry::Enum(inner) => {
                    handle_enum(inner, depth + 1, o);
                }
                Entry::Message(inner) => {
                    handle_message(inner, pbv, &ctx, depth + 1, o);
                }
                Entry::Oneof(oneof) => {
                    push_indent(depth + 1, o);
                    o.extend(b"#[derive(Clone, PartialEq, ::prost::Oneof)]\n");
                    push_indent(depth + 1, o);
                    o.extend(b"pub enum ");
                    push_big_camel(oneof.name, o);
                    o.extend(b" {\n");
                    for field in &oneof.fields {
                        // attr macros
                        push_indent(depth + 2, o);
                        o.extend(b"#[prost(");
                        if to_rust_type(field.data_type) == b"custom" {
                            o.extend(b"message");
                        } else {
                            o.extend(field.data_type);
                        }
                        o.extend(b", tag=\"");
                        o.extend(field.tag);
                        o.extend(b"\")]\n");

                        // value
                        push_indent(depth + 2, o);
                        push_big_camel(field.name, o);
                        o.extend(b"(");
                        match to_rust_type(field.data_type) {
                            b"custom" => {
                                o.extend(b"super::");
                                push_big_camel(field.data_type, o);
                            }
                            v => o.extend(v),
                        }
                        o.extend(b"),\n");
                    }
                    push_indent(depth + 1, o);
                    o.extend(b"}\n");
                }
                Entry::MessageField(_) => {}
            }
        }
        push_indent(depth, o);
        o.extend(b"}\n");
    }

    fn handle_enum(enume: &Enum, mut depth: i32, o: &mut Vec<u8>) {
        push_indent(depth, o);
        o.extend(b"#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]\n");
        push_indent(depth, o);
        o.extend(b"#[repr(i32)]\n");
        push_indent(depth, o);
        o.extend(b"pub enum ");
        push_big_camel(enume.name, o);
        o.extend(b" {\n");
        depth += 1;
        for field in &enume.fields {
            push_indent(depth, o);
            push_big_camel(field.name, o);
            o.extend(b" = ");
            push_big_camel(field.tag, o);
            o.extend(b",\n");
        }
        depth -= 1;
        push_indent(depth, o);
        o.extend(b"}\n");
    }

    let depth = -1;
    for entry in &package.entries {
        match entry {
            Entry::Enum(inner) => {
                ctx.insert(inner.name, (b"enum", depth));
            }
            Entry::Message(_) => {}
            _ => unreachable!(),
        }
    }
    for entry in &package.entries {
        match entry {
            Entry::Enum(inner) => {
                handle_enum(inner, depth + 1, &mut o);
            }
            Entry::Message(inner) => {
                handle_message(inner, package.syntax, &ctx, depth + 1, &mut o);
            }
            _ => unreachable!(),
        }
    }
    o
}

/// Compile `.proto` files into Rust files during a Cargo build.
pub fn compile_protos(
    protos: &[impl AsRef<Path>],
    _includes: &[impl AsRef<Path>],
) -> std::io::Result<()> {
    let mut outs = HashMap::<Vec<u8>, Vec<u8>>::new();
    // let begin_instant = std::time::Instant::now();
    for path in protos {
        // dbg!(path.as_ref());
        let src = std::fs::read(path)?;
        let token_stream = TokenStream::new(&src);
        let package = Package::new(&token_stream);
        let name = package.name.to_vec();
        let mut out = translate(&package);
        if let Some(existed) = outs.get_mut(&name) {
            existed.append(&mut out);
        } else {
            outs.insert(name, out);
        }
    }
    // println!("{} ms", begin_instant.elapsed().as_micros() as f64 / 1000.0);
    for (name, out) in outs {
        std::fs::write(
            format!(
                "{}/{}.rs",
                std::env::var("OUT_DIR").unwrap(),
                String::from_utf8(name).unwrap()
            ),
            out,
        )?;
    }
    Ok(())
}

/// # DON'T USE THIS IN LIB TARGET!
pub fn main() {
    // return test_to_any_case();

    const IN_DIR: &str = "./1_in";
    const OUT_DIR: &str = "./2_out";
    let mut v = Vec::new();
    recurse_dir(&mut v, IN_DIR);
    fn recurse_dir(v: &mut Vec<PathBuf>, dir: impl AsRef<Path>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                recurse_dir(v, path);
            } else if let Some(true) = path.extension().map(|v| v == "proto") {
                v.push(path);
            }
        }
    }

    std::fs::remove_dir_all(OUT_DIR).ok();
    std::fs::create_dir_all(OUT_DIR).unwrap();
    std::env::set_var("OUT_DIR", OUT_DIR);

    // prost_build_offical::compile_protos(&v, &[IN_DIR]).unwrap();
    compile_protos(&v, &[IN_DIR]).unwrap();
}
