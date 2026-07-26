use cbus_core::{
    Subscriber,
    buffer::{IncBuf, OutBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub struct EmptySubscriber<const M: usize> {
    _inc: IncBuf<M>,
    _out: OutBuf<M>,
}
impl<const M: usize> EmptySubscriber<M> {
    pub fn new(inc: IncBuf<M>, out: OutBuf<M>) -> Self {
        Self {
            _inc: inc,
            _out: out,
        }
    }
}

impl<const M: usize> Subscriber for EmptySubscriber<M> {
    fn notify(&mut self) {}
}
