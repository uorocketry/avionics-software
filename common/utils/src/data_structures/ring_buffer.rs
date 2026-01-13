use core::array::from_fn;

pub enum BufferError {
    BufferEmpty,
    BufferOverrun,
}
pub struct RingBuffer<T, const N: usize> {
    internal_buffer: [T; N],
    read_index: usize,
    write_index: usize,
}
impl<T: Default + Clone, const N: usize> RingBuffer<T, N> {
    pub fn new() -> RingBuffer<T, N>{
        RingBuffer { internal_buffer: from_fn(|_| {T::default()}), read_index: 0, write_index: 0}
    }

    pub fn push(&mut self, item: T) -> Result<(), BufferError>{
        self.internal_buffer[self.write_index] = item;
        self.write_index = (self.write_index + 1) % N;

        // Checks if the ring buffer has been overrun
        if self.write_index < self.read_index {
            self.read_index =  (self.read_index + 1) % N;
            return Err(BufferError::BufferOverrun);
        }
        return Ok(());
    }

    pub fn pop(&mut self) -> Result<T, BufferError> {
        if self.read_index == self.write_index {
            return Err(BufferError::BufferEmpty);
        }
        let result = self.internal_buffer[self.read_index].clone();
        self.read_index = (self.read_index + 1) % N;

        return Ok(result);
    }
}