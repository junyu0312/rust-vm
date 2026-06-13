pub struct Ring<const SIZE: usize, T> {
    buffer: [T; SIZE],
    write_index: usize,
    read_index: usize,
}

impl<const SIZE: usize, T> Default for Ring<SIZE, T>
where
    T: Copy + Default,
{
    fn default() -> Self {
        Self {
            buffer: [T::default(); SIZE],
            write_index: 0,
            read_index: 0,
        }
    }
}

impl<const SIZE: usize, T> Ring<SIZE, T>
where
    T: Copy + Default,
{
    #[inline]
    fn next(i: usize) -> usize {
        (i + 1) % SIZE
    }

    pub fn is_empty(&self) -> bool {
        self.read_index == self.write_index
    }

    pub fn is_full(&self) -> bool {
        Self::next(self.write_index) == self.read_index
    }

    pub fn push(&mut self, v: T) -> Result<(), T> {
        if self.is_full() {
            return Err(v);
        }

        self.buffer[self.write_index] = v;
        self.write_index = Self::next(self.write_index);

        Ok(())
    }

    pub fn try_pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let v = self.buffer[self.read_index];
        self.read_index = Self::next(self.read_index);

        Some(v)
    }

    pub fn clean(&mut self) {
        self.write_index = 0;
        self.read_index = 0;
    }
}
