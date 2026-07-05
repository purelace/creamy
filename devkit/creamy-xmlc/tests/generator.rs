#![allow(unused)]

#[derive(Default)]
pub struct XMLGeneratorBuilder {
    groups: usize,
    messages_per_group: usize,
    structs_per_group: usize,
    enums_per_group: usize,
    fields_per_message: usize,
    fields_per_struct: usize,
    variants_per_enum: usize,
}

impl XMLGeneratorBuilder {
    #[must_use]
    pub const fn groups(mut self, val: usize) -> Self {
        self.groups = val;
        self
    }

    #[must_use]
    pub const fn messages_per_group(mut self, val: usize) -> Self {
        self.messages_per_group = val;
        self
    }

    #[must_use]
    pub const fn structs_per_group(mut self, val: usize) -> Self {
        self.structs_per_group = val;
        self
    }

    #[must_use]
    pub const fn enums_per_group(mut self, val: usize) -> Self {
        self.enums_per_group = val;
        self
    }

    #[must_use]
    pub const fn fields_per_message(mut self, val: usize) -> Self {
        self.fields_per_message = val;
        self
    }

    #[must_use]
    pub const fn fields_per_struct(mut self, val: usize) -> Self {
        self.fields_per_struct = val;
        self
    }

    #[must_use]
    pub const fn variants_per_enum(mut self, val: usize) -> Self {
        self.variants_per_enum = val;
        self
    }

    #[must_use]
    pub const fn build(self) -> XMLGenerator {
        XMLGenerator {
            config: self,
            groups_created: 0,
            messages_created: 0,
            structs_created: 0,
            enums_created: 0,
            child_created: 0,
            state: State::OpenGroup,
        }
    }
}

pub struct XMLGenerator {
    config: XMLGeneratorBuilder,
    groups_created: usize,
    messages_created: usize,
    structs_created: usize,
    enums_created: usize,
    child_created: usize, // Счетчик полей/вариантов внутри текущего айтема
    state: State,
}

enum State {
    OpenGroup,
    OpenMessage,
    MessageFields,
    CloseMessage,
    OpenStruct,
    StructFields,
    CloseStruct,
    OpenEnum,
    EnumVariants,
    CloseEnum,
    CloseGroup,
}

impl Iterator for XMLGenerator {
    type Item = String;

    #[allow(clippy::too_many_lines)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.groups_created >= self.config.groups {
            return None;
        }

        match self.state {
            State::OpenGroup => {
                self.state = State::OpenMessage;
                Some(format!(
                    r#"<group name="Group{}" access="Protected">"#,
                    self.groups_created
                ))
            }
            // --- MESSAGES ---
            State::OpenMessage => {
                let limit = (self.groups_created + 1) * self.config.messages_per_group;
                if self.messages_created < limit {
                    self.child_created = 0;
                    self.state = State::MessageFields;
                    Some(format!(
                        r#"<message name="Message{}">"#,
                        self.messages_created
                    ))
                } else {
                    self.state = State::OpenStruct;
                    self.next()
                }
            }
            State::MessageFields => {
                if self.child_created < self.config.fields_per_message {
                    let res = format!(r#"<field name="f{}" type="u8" />"#, self.child_created);
                    self.child_created += 1;
                    Some(res)
                } else {
                    self.state = State::CloseMessage;
                    self.next()
                }
            }
            State::CloseMessage => {
                self.messages_created += 1;
                self.state = State::OpenMessage;
                Some("</message>".to_string())
            }
            // --- STRUCTS ---
            State::OpenStruct => {
                let limit = (self.groups_created + 1) * self.config.structs_per_group;
                if self.structs_created < limit {
                    self.child_created = 0;
                    self.state = State::StructFields;
                    Some(format!(r#"<struct name="Struct{}">"#, self.structs_created))
                } else {
                    self.state = State::OpenEnum;
                    self.next()
                }
            }
            State::StructFields => {
                if self.child_created < self.config.fields_per_struct {
                    let res = format!(r#"<field name="s{}" type="u8" />"#, self.child_created);
                    self.child_created += 1;
                    Some(res)
                } else {
                    self.state = State::CloseStruct;
                    self.next()
                }
            }
            State::CloseStruct => {
                self.structs_created += 1;
                self.state = State::OpenStruct;
                Some("</struct>".to_string())
            }
            // --- ENUMS ---
            State::OpenEnum => {
                let limit = (self.groups_created + 1) * self.config.enums_per_group;
                if self.enums_created < limit {
                    self.child_created = 0;
                    self.state = State::EnumVariants;
                    Some(format!(
                        r#"<enum name="Enum{}" repr="u8">"#,
                        self.enums_created
                    ))
                } else {
                    self.state = State::CloseGroup;
                    self.next()
                }
            }
            State::EnumVariants => {
                if self.child_created < self.config.variants_per_enum {
                    let res = format!(
                        r#"<variant name="V{}" value="{}" />"#,
                        self.child_created, self.child_created
                    );
                    self.child_created += 1;
                    Some(res)
                } else {
                    self.state = State::CloseEnum;
                    self.next()
                }
            }
            State::CloseEnum => {
                self.enums_created += 1;
                self.state = State::OpenEnum;
                Some("</enum>".to_string())
            }
            // --- GROUP END ---
            State::CloseGroup => {
                self.groups_created += 1;
                self.state = State::OpenGroup;
                Some("</group>".to_string())
            }
        }
    }
}
