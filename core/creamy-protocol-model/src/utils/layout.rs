use crate::symbols::{Field, FieldSymbol, FieldType};

#[derive(Debug, Copy, Clone)]
pub struct LayoutStep {
    pub size: u8,
    pub align: u8,
    pub padding: u8,
    pub offset: u8,
}

impl LayoutStep {
    // TODO: const
    fn new<F: Field>(symbol: &F, total_size: u8) -> Self {
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

#[derive(Clone)]
pub struct LayoutCalculator<F: Field, I: Iterator<Item = F>> {
    total_size: u8,
    fields: I,
}

impl<F: Field, I: Iterator<Item = F>> LayoutCalculator<F, I> {
    #[must_use]
    pub const fn new(reserved: u8, fields: I) -> Self {
        Self {
            total_size: reserved,
            fields,
        }
    }

    #[must_use]
    pub const fn total_size(&self) -> u8 {
        self.total_size
    }
}

impl<F: Field, I: Iterator<Item = F>> Iterator for LayoutCalculator<F, I> {
    type Item = (F, LayoutStep);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(field) = self.fields.next() {
            let step = LayoutStep::new(&field, self.total_size);

            //let Some(result) = self.total_size.checked_add(step.padding + step.size) else {
            //
            //};

            //TODO: overflow
            self.total_size += step.padding + step.size;

            Some((field, step))
        } else {
            None
        }
    }
}

#[allow(clippy::missing_errors_doc)]
pub fn compute_layout(
    reserved: u8,
    fields: &[FieldSymbol],
    mut f: impl FnMut(FieldSymbol, LayoutStep),
) -> u8 {
    let calculator = LayoutCalculator::new(reserved, fields.iter().copied());
    let mut total_size = reserved;

    for (field, step) in calculator {
        total_size = step.offset + step.size;
        f(field, step);
    }
    total_size
}
