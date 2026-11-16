#[derive(Default)]
pub struct CMPGeneratorBuilder {
    messages: usize,
    fields_per_message: usize,
}

impl CMPGeneratorBuilder {
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
    pub const fn build(self) -> Generator {
        Generator {
            config: self,
            messages_created: 0,
            child_created: 0,
            state: State::OpenMessage,
        }
    }
}

pub struct Generator {
    config: CMPGeneratorBuilder,
    messages_created: usize,
    child_created: usize,
    state: State,
}

enum State {
    OpenMessage,
    MessageFields,
    CloseMessage,
}

impl Iterator for Generator {
    type Item = String;

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
                        "message[D] Message{value} {{\n",
                        value = self.messages_created,
                    ))
                } else {
                    self.state = State::CloseMessage;
                    self.next()
                }
            }
            State::MessageFields => {
                if self.child_created < self.config.fields_per_message {
                    let res = format!("f{}: u8,", self.child_created);
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
                Some("\n}".to_string())
            }
        }
    }
}
