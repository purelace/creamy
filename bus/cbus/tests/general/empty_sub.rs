use cbus_core::Subscriber;

pub struct EmptySubscriber;
impl Subscriber for EmptySubscriber {
    fn notify(&mut self) {}
}
