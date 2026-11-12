#[derive(Default)]
pub struct XMLGeneratorBuilder {
    messages: usize,
    fields_per_message: usize,
}

impl XMLGeneratorBuilder {
    #[must_use]
    pub const fn messages(mut self, val: usize) -> Self {
        self.messages = val;
        self
    }

    #[must_use]
    pub const fn fields_per_message(mut self, val: usize) -> Self {
        self.fields_per_message = val;
        self
    }

    #[must_use]
    pub const fn build(self) -> XMLGenerator {
        XMLGenerator {
            config: self,
            messages_created: 0,
            child_created: 0,
            state: State::OpenMessage,
        }
    }
}

pub struct XMLGenerator {
    config: XMLGeneratorBuilder,
    messages_created: usize,
    child_created: usize, // Счетчик полей/вариантов внутри текущего айтема
    state: State,
}

enum State {
    OpenMessage,
    MessageFields,
    CloseMessage,
}

impl Iterator for XMLGenerator {
    type Item = String;

    #[allow(clippy::too_many_lines)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.messages_created >= self.config.messages {
            return None;
        }

        match self.state {
            // --- MESSAGES ---
            State::OpenMessage => {
                let limit = self.config.messages;
                if self.messages_created < limit {
                    self.child_created = 0;
                    self.state = State::MessageFields;
                    Some(format!(
                        r#"<message name="Message{value}" direction="Duplex">"#,
                        value = self.messages_created,
                    ))
                } else {
                    self.state = State::CloseMessage;
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
        }
    }
}
