use core::array::from_fn;

pub enum QueueError {
    QueueEmpty,
    QueueOverrun,
}
pub struct Queue<T, const N: usize> {
    internal_buffer: [T; N],
    index: usize,
}
impl<T: Default + Clone, const N: usize> Queue<T, N> {
    pub fn new() -> Queue<T, N>{
        Queue { internal_buffer: from_fn(|_| {T::default()}), index: 0 }
    }

    pub fn push(&mut self, item: T) -> Result<(), QueueError>{
        if self.index == N {
            // Overrite the data last added to queue to maintain coherence
            self.internal_buffer[self.index - 1] = item;
            return Err(QueueError::QueueOverrun);
        }
        self.internal_buffer[self.index] = item;
        self.index += 1; 
        return Ok(());
    }

    pub fn pop(&mut self) -> Result<T, QueueError> {
        if self.index < 1 {
            return Err(QueueError::QueueEmpty);
        }
        let result = self.internal_buffer[0].clone();
        
        for i in 0..self.index - 1 {
            self.internal_buffer[i] = self.internal_buffer[i + 1].clone();
        }
        self.index -= 1;

        return Ok(result);
    }
}