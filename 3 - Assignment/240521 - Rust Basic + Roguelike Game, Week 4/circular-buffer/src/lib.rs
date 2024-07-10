#[cfg(test)]
use std::rc::Rc;

pub struct CircularBuffer<T> {
    buffer: Vec<Option<T>>,
    start_idx: usize,
    end_idx: usize,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyBuffer,
    FullBuffer,
}

impl<T> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: (0..capacity).map(|_| None).collect(),
            start_idx: 0,
            end_idx: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.start_idx == self.end_idx && self.buffer[self.start_idx].is_none()
    }

    fn is_full(&self) -> bool {
        self.start_idx == self.end_idx && self.buffer[self.start_idx].is_some()
    }

    pub fn write(&mut self, element: T) -> Result<(), Error> {
        if self.is_full() {
            Err(Error::FullBuffer)
        } else {
            let capacity = self.buffer.len();
            self.buffer[self.end_idx] = Some(element);
            self.end_idx = (self.end_idx + 1) % capacity;
            Ok(())
        }
    }

    pub fn read(&mut self) -> Result<T, Error> {
        if self.is_empty() {
            Err(Error::EmptyBuffer)
        } else {
            let capacity = self.buffer.len();
            let value = std::mem::replace(&mut self.buffer[self.start_idx], None);
            self.start_idx = (self.start_idx + 1) % capacity;
            Ok(value.unwrap())
        }
    }

    pub fn clear(&mut self) {
        for idx in 0..self.buffer.len() {
            self.buffer[idx] = None;
        }
    }

    pub fn overwrite(&mut self, element: T) {
        let capacity = self.buffer.len();
        self.buffer[self.end_idx] = Some(element);
        if self.start_idx == self.end_idx {
            self.start_idx = (self.start_idx + 1) % capacity;
        }
        self.end_idx = (self.end_idx + 1) % capacity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_on_read_empty_buffer() {
        let mut buffer = CircularBuffer::<char>::new(1);
        assert_eq!(Err(Error::EmptyBuffer), buffer.read());
    }

    #[test]
    fn can_read_item_just_written() {
        let mut buffer = CircularBuffer::new(1);
        assert!(buffer.write('1').is_ok());
        assert_eq!(Ok('1'), buffer.read());
    }

    #[test]
    fn each_item_may_only_be_read_once() {
        let mut buffer = CircularBuffer::new(1);
        assert!(buffer.write('1').is_ok());
        assert_eq!(Ok('1'), buffer.read());
        assert_eq!(Err(Error::EmptyBuffer), buffer.read());
    }

    #[test]
    fn items_are_read_in_the_order_they_are_written() {
        let mut buffer = CircularBuffer::new(2);
        assert!(buffer.write('1').is_ok());
        assert!(buffer.write('2').is_ok());
        assert_eq!(Ok('1'), buffer.read());
        assert_eq!(Ok('2'), buffer.read());
        assert_eq!(Err(Error::EmptyBuffer), buffer.read());
    }

    #[test]
    fn full_buffer_cant_be_written_to() {
        let mut buffer = CircularBuffer::new(1);
        assert!(buffer.write('1').is_ok());
        assert_eq!(Err(Error::FullBuffer), buffer.write('2'));
    }

    #[test]
    fn read_frees_up_capacity_for_another_write() {
        let mut buffer = CircularBuffer::new(1);
        assert!(buffer.write('1').is_ok());
        assert_eq!(Ok('1'), buffer.read());
        assert!(buffer.write('2').is_ok());
        assert_eq!(Ok('2'), buffer.read());
    }

    #[test]
    fn read_position_is_maintained_even_across_multiple_writes() {
        let mut buffer = CircularBuffer::new(3);
        assert!(buffer.write('1').is_ok());
        assert!(buffer.write('2').is_ok());
        assert_eq!(Ok('1'), buffer.read());
        assert!(buffer.write('3').is_ok());
        assert_eq!(Ok('2'), buffer.read());
        assert_eq!(Ok('3'), buffer.read());
    }

    #[test]
    fn items_cleared_out_of_buffer_cant_be_read() {
        let mut buffer = CircularBuffer::new(1);
        assert!(buffer.write('1').is_ok());
        buffer.clear();
        assert_eq!(Err(Error::EmptyBuffer), buffer.read());
    }

    #[test]
    fn clear_frees_up_capacity_for_another_write() {
        let mut buffer = CircularBuffer::new(1);
        assert!(buffer.write('1').is_ok());
        buffer.clear();
        assert!(buffer.write('2').is_ok());
        assert_eq!(Ok('2'), buffer.read());
    }

    #[test]
    fn clear_does_nothing_on_empty_buffer() {
        let mut buffer = CircularBuffer::new(1);
        buffer.clear();
        assert!(buffer.write('1').is_ok());
        assert_eq!(Ok('1'), buffer.read());
    }

    #[test]
    fn clear_actually_frees_up_its_elements() {
        let mut buffer = CircularBuffer::new(1);
        let element = Rc::new(());
        assert!(buffer.write(Rc::clone(&element)).is_ok());
        assert_eq!(Rc::strong_count(&element), 2);
        buffer.clear();
        assert_eq!(Rc::strong_count(&element), 1);
    }

    #[test]
    fn overwrite_acts_like_write_on_non_full_buffer() {
        let mut buffer = CircularBuffer::new(2);
        assert!(buffer.write('1').is_ok());
        buffer.overwrite('2');
        assert_eq!(Ok('1'), buffer.read());
        assert_eq!(Ok('2'), buffer.read());
        assert_eq!(Err(Error::EmptyBuffer), buffer.read());
    }

    #[test]
    fn overwrite_replaces_the_oldest_item_on_full_buffer() {
        let mut buffer = CircularBuffer::new(2);
        assert!(buffer.write('1').is_ok());
        assert!(buffer.write('2').is_ok());
        buffer.overwrite('A');
        assert_eq!(Ok('2'), buffer.read());
        assert_eq!(Ok('A'), buffer.read());
    }

    #[test]
    fn overwrite_replaces_the_oldest_item_remaining_in_buffer_following_a_read() {
        let mut buffer = CircularBuffer::new(3);
        assert!(buffer.write('1').is_ok());
        assert!(buffer.write('2').is_ok());
        assert!(buffer.write('3').is_ok());
        assert_eq!(Ok('1'), buffer.read());
        assert!(buffer.write('4').is_ok());
        buffer.overwrite('5');
        assert_eq!(Ok('3'), buffer.read());
        assert_eq!(Ok('4'), buffer.read());
        assert_eq!(Ok('5'), buffer.read());
    }

    #[test]
    fn integer_buffer() {
        let mut buffer = CircularBuffer::new(2);
        assert!(buffer.write(1).is_ok());
        assert!(buffer.write(2).is_ok());
        assert_eq!(Ok(1), buffer.read());
        assert!(buffer.write(-1).is_ok());
        assert_eq!(Ok(2), buffer.read());
        assert_eq!(Ok(-1), buffer.read());
        assert_eq!(Err(Error::EmptyBuffer), buffer.read());
    }

    #[test]
    fn string_buffer() {
        let mut buffer = CircularBuffer::new(2);
        buffer.write("".to_string()).unwrap();
        buffer.write("Testing".to_string()).unwrap();
        assert_eq!(0, buffer.read().unwrap().len());
        assert_eq!(Ok("Testing".to_string()), buffer.read());
    }
}
