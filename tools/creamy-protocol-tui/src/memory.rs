use std::borrow::Cow;

use creamy_xmlc::model::constraints::HEADER_BYTES;
use ptree::{Color, PrintConfig, Style, TreeItem, print_config::UTF_CHARS, print_tree_with};

const fn style(color: Color) -> Style {
    Style {
        foreground: Some(color),
        background: None,
        bold: true,
        dimmed: false,
        italic: false,
        underline: false,
        blink: false,
        reverse: false,
        hidden: false,
        strikethrough: false,
    }
}

const STRUCT_STYLE: Style = style(Color::RGB(78, 201, 176));
const GOOD_STYLE: Style = style(Color::Green);
const BAD_STYLE: Style = style(Color::Red);
const NUMBER_STYLE: Style = style(Color::RGB(181, 206, 168));
const BRACKET_STYLE: Style = style(Color::Blue);
const NORMAL_NAME_STYLE: Style = style(Color::RGB(156, 220, 254));
const PADDING_NAME_STYLE: Style = style(Color::Red);

#[derive(Clone)]
pub struct ArrayField {
    pub name: String,
    pub kind: String,
    pub size: usize,
    pub align: usize,
    pub is_padding: bool,
}

impl TreeItem for ArrayField {
    type Child = Self;

    fn write_self<W: std::io::Write>(&self, f: &mut W, _style: &Style) -> std::io::Result<()> {
        let (name, asterisk) = if self.is_padding {
            (PADDING_NAME_STYLE.paint(&self.name), "*")
        } else {
            (NORMAL_NAME_STYLE.paint(&self.name), "")
        };

        let kind = STRUCT_STYLE.paint(&self.kind);
        let size = NUMBER_STYLE.paint(self.size);
        let l_brac = BRACKET_STYLE.paint('[');
        let r_brac = BRACKET_STYLE.paint(']');
        write!(
            f,
            "[{}:{}] {name}: {l_brac}{kind}; {size}{r_brac} {asterisk}",
            self.size, self.align
        )
    }

    fn children(&self) -> Cow<'_, [Self::Child]> {
        Cow::default()
    }
}

#[derive(Clone)]
pub struct StructReport {
    name: String,
    total_size: usize,
    padding_size: usize,
    align: usize,
    fields: Vec<Field>,
}

impl StructReport {
    fn config() -> PrintConfig {
        let mut config = PrintConfig {
            depth: u32::MAX,
            indent: 4,
            characters: UTF_CHARS.into(),
            ..Default::default()
        };

        config.branch = Style {
            foreground: Some(Color::RGB(128, 128, 128)),
            ..Style::default()
        };

        config
    }

    pub fn print_tree(&self) {
        print_tree_with(self, &Self::config()).unwrap();
    }
}

impl TreeItem for StructReport {
    type Child = Field;

    fn write_self<W: std::io::Write>(&self, f: &mut W, _style: &Style) -> std::io::Result<()> {
        let name = STRUCT_STYLE.paint(&self.name);
        let used = GOOD_STYLE.paint(self.total_size + HEADER_BYTES as usize);
        let align = GOOD_STYLE.paint(self.align);
        let padding = if self.padding_size == 0 {
            GOOD_STYLE
        } else {
            BAD_STYLE
        }
        .paint(self.padding_size);
        write!(f, "— {name} [{used}:{align}] [{padding}]")
    }

    fn children(&self) -> Cow<'_, [Self::Child]> {
        Cow::Borrowed(&self.fields)
    }
}

#[derive(Clone)]
pub enum Field {
    Simple(SimpleField),
    Complex(ComplexField),
    Array(ArrayField),
}

impl TreeItem for Field {
    type Child = Self;

    fn write_self<W: std::io::Write>(&self, f: &mut W, style: &Style) -> std::io::Result<()> {
        match self {
            Field::Simple(field) => field.write_self(f, style),
            Field::Complex(field) => field.write_self(f, style),
            Field::Array(field) => field.write_self(f, style),
        }
    }

    fn children(&self) -> Cow<'_, [Self::Child]> {
        match self {
            Field::Complex(field) => field.children(),
            Field::Simple(_) | Field::Array(_) => Cow::default(),
        }
    }
}

#[derive(Clone)]
pub struct SimpleField {
    pub name: String,
    pub kind: String,
    pub size: usize,
    pub align: usize,
}

impl TreeItem for SimpleField {
    type Child = Self;

    fn write_self<W: std::io::Write>(&self, f: &mut W, _style: &Style) -> std::io::Result<()> {
        let name = NORMAL_NAME_STYLE.paint(&self.name);
        let kind = STRUCT_STYLE.paint(&self.kind);
        write!(f, "[{}:{}] {name}: {kind}", self.size, self.align)
    }

    fn children(&self) -> Cow<'_, [Self::Child]> {
        Cow::default()
    }
}

#[derive(Clone)]
pub struct ComplexField {
    pub name: String,
    pub kind: String,
    pub size: usize,
    pub align: usize,
    pub fields: Vec<Field>,
}

impl TreeItem for ComplexField {
    type Child = Field;

    fn write_self<W: std::io::Write>(&self, f: &mut W, _style: &Style) -> std::io::Result<()> {
        let name = NORMAL_NAME_STYLE.paint(&self.name);
        let kind = STRUCT_STYLE.paint(&self.kind);
        write!(f, "[{}:{}] {name}: {kind}", self.size, self.align)
    }

    fn children(&self) -> Cow<'_, [Self::Child]> {
        Cow::Borrowed(&self.fields)
    }
}

pub struct MemoryReport {
    source: String,
    stack: Vec<Vec<Field>>,
    current: Vec<Field>,
}

impl MemoryReport {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            stack: vec![],
            current: vec![],
        }
    }

    pub fn start_report(&mut self) {
        let previous = std::mem::take(&mut self.current);
        self.stack.push(previous);
    }

    pub fn add_field(&mut self, field: SimpleField) {
        self.current.push(Field::Simple(field));
    }

    pub fn add_array_field(&mut self, field: ArrayField) {
        self.current.push(Field::Array(field));
    }

    pub fn end_report(&mut self, field: SimpleField) {
        let field = Field::Complex(ComplexField {
            name: field.name,
            kind: field.kind,
            size: field.size,
            align: field.align,
            fields: std::mem::take(&mut self.current),
        });
        self.current = self.stack.pop().unwrap_or_default();
        self.current.push(field);
    }

    pub fn finish(self, size: usize, align: usize, padding_size: usize) -> StructReport {
        StructReport {
            name: self.source,
            total_size: size,
            padding_size,
            align,
            fields: self.current,
        }
    }
}
