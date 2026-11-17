use super::Size;
use crate::symbols::{FieldSymbol, T_U8_ID};

#[derive(Debug, PartialEq, Eq)]
pub enum LayoutItem {
    Field(FieldSymbol),
    Padding { index: u8, size: Size },
}

impl LayoutItem {
    fn align(&self) -> u64 {
        match self {
            LayoutItem::Field(f) => f.align(),
            LayoutItem::Padding { .. } => T_U8_ID.align().size(),
        }
    }

    fn size(&self) -> u64 {
        match self {
            LayoutItem::Field(f) => f.size(),
            LayoutItem::Padding { size, .. } => size.value(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LayoutStep {
    pub size: u64,
    pub align: u64,
    pub padding: u64,
    pub offset: u64,
}

impl LayoutStep {
    fn new(symbol: &LayoutItem, total_size: u64) -> Self {
        let align = symbol.align();
        let size = symbol.size();

        let padding = (align - (total_size % align)) % align;
        Self {
            size,
            align,
            padding,
            offset: total_size + padding,
        }
    }
}

pub struct LayoutCalculator<I: Iterator<Item = FieldSymbol>> {
    total_size: u64,
    padding_index: u8,
    max_align: u64,
    left_pad: bool,
    fields: I,
    prev: Option<(LayoutItem, LayoutStep)>,
}

impl<I: Iterator<Item = FieldSymbol>> LayoutCalculator<I> {
    #[must_use]
    pub fn new(max_align: Option<u8>, fields: I) -> Self {
        Self {
            total_size: 0,
            padding_index: 0,
            max_align: max_align.map_or(1, u64::from),
            left_pad: false,
            fields,
            prev: None,
        }
    }

    #[must_use]
    pub const fn total_size(&self) -> u64 {
        self.total_size
    }

    fn new_padding(&mut self, padding: u64) -> (LayoutItem, LayoutStep) {
        let size = Size::try_new(padding).unwrap();
        let item = LayoutItem::Padding {
            index: self.padding_index,
            size,
        };
        self.padding_index += 1;

        let step = LayoutStep::new(&item, self.total_size);
        self.total_size += step.size;

        (item, step)
    }
}

impl<I: Iterator<Item = FieldSymbol>> Iterator for LayoutCalculator<I> {
    type Item = (LayoutItem, LayoutStep);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((f, s)) = self.prev.take() {
            self.total_size += s.size;
            return Some((f, s));
        }

        if let Some(field) = self.fields.next() {
            let item = LayoutItem::Field(field);
            let step = LayoutStep::new(&item, self.total_size);
            self.max_align = self.max_align.max(step.align);

            return if step.padding == 0 {
                self.total_size += step.size;
                Some((item, step))
            } else {
                self.prev = Some((item, step));
                Some(self.new_padding(step.padding))
            };
        }

        if self.left_pad {
            return None;
        }
        self.left_pad = true;

        let left = (self.max_align - (self.total_size % self.max_align)) % self.max_align;
        if left == 0 {
            None
        } else {
            Some(self.new_padding(left))
        }
    }
}

pub fn compute_layout(
    max_align: Option<u8>,
    fields: &[FieldSymbol],
    mut f: impl FnMut(LayoutItem, LayoutStep),
) -> u64 {
    let calculator = LayoutCalculator::new(max_align, fields.iter().copied());
    let mut total_size = 0;

    for (field, step) in calculator {
        total_size = step.offset + step.size;
        f(field, step);
    }
    total_size
}
