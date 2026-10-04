#[derive(Debug, Default)]
pub struct Mailbox<T> {
    slot: Option<T>,
}

impl<T> Mailbox<T> {
    pub fn publish(&mut self, value: T) {
        self.slot = Some(value);
    }

    pub fn latest(&mut self) -> Option<T> {
        self.slot.take()
    }
}
