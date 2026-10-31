use super::Size;
use crate::{
    TypeId,
    symbols::{Field, FieldSymbol, T_U8_ID},
};

#[derive(Debug, PartialEq, Eq)]
pub enum LayoutItem<F: Field> {
    Field(F),
    Padding { index: u8, size: Size },
}

impl<F: Field> LayoutItem<F> {
    fn type_id(&self) -> TypeId {
        match self {
            LayoutItem::Field(f) => f.type_id(),
            LayoutItem::Padding { .. } => T_U8_ID,
        }
    }

    fn size(&self) -> u8 {
        match self {
            LayoutItem::Field(f) => f.size(),
            LayoutItem::Padding { size, .. } => size.value(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LayoutStep {
    pub size: u8,
    pub align: u8,
    pub padding: u8,
    pub offset: u8,
}

impl LayoutStep {
    fn new<F: Field>(symbol: &LayoutItem<F>, total_size: u8) -> Self {
        let type_id = symbol.type_id();
        let align = type_id.align().size();
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

pub struct LayoutCalculator<F: Field, I: Iterator<Item = F>> {
    total_size: u8,
    padding_index: u8,
    max_align: u8,
    left_pad: bool,
    fields: I,
    prev: Option<(LayoutItem<F>, LayoutStep)>,
}

impl<F: Field, I: Iterator<Item = F>> LayoutCalculator<F, I> {
    #[must_use]
    pub fn new(reserved: u8, max_align: Option<u8>, fields: I) -> Self {
        Self {
            total_size: reserved,
            padding_index: 0,
            max_align: max_align.unwrap_or(1),
            left_pad: false,
            fields,
            prev: None,
        }
    }

    #[must_use]
    pub const fn total_size(&self) -> u8 {
        self.total_size
    }

    fn new_padding(&mut self, padding: u8) -> (LayoutItem<F>, LayoutStep) {
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

impl<F: Field, I: Iterator<Item = F>> Iterator for LayoutCalculator<F, I> {
    type Item = (LayoutItem<F>, LayoutStep);

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

            //let Some(result) = self.total_size.checked_add(step.padding + step.size) else {
            //
            //};
            //TODO: overflow
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
    reserved: u8,
    max_align: Option<u8>,
    fields: &[FieldSymbol],
    mut f: impl FnMut(LayoutItem<FieldSymbol>, LayoutStep),
) -> u8 {
    let calculator = LayoutCalculator::new(reserved, max_align, fields.iter().copied());
    let mut total_size = reserved;

    for (field, step) in calculator {
        total_size = step.offset + step.size;
        f(field, step);
    }
    total_size
}
