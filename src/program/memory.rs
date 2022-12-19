use std::fmt::Debug;

#[derive(Debug)]
pub struct Memory {
    memory: Vec<u8>,
    free: Vec<(usize, usize)>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            memory: Vec::new(),
            free: vec![(0, usize::MAX)],
        }
    }
    pub fn push(&mut self, value: u8) -> usize {
        let (address, remaining) = self.free.get_mut(0).unwrap();
        let starting_address = *address;
        self.memory.resize(self.memory.len().max(*address + 1), 0); // extend memory;
        self.memory[*address] = value;
        *address += 1;
        *remaining -= 1;
        if *remaining == 0 {
            self.free.remove(0);
        };
        starting_address
    }
    pub fn extend(&mut self, data: &[u8]) -> usize {
        let index = self
            .free
            .iter()
            .position(|&(_address, free)| free >= data.len())
            .unwrap();
        let (address, remaining) = self.free.get_mut(index).unwrap();
        let starting_address = *address;
        self.memory
            .resize(self.memory.len().max(*address + data.len()), 0); // extend memory;
        *remaining -= data.len();

        for value in data {
            self.memory[*address] = *value;
            *address += 1;
        }
        if *remaining == 0 {
            self.free.remove(0);
        }
        starting_address
    }
    pub fn alloc(&mut self, len: usize) -> usize {
        let index = self
            .free
            .iter()
            .position(|&(_address, free)| free >= len)
            .unwrap();
        let (address, remaining) = self.free.get_mut(index).unwrap();
        let starting_address = *address;
        self.memory.resize(self.memory.len().max(*address + len), 0); // extend memory;
        *remaining -= len;

        if *remaining == 0 {
            self.free.remove(0);
        }
        starting_address
    }
    #[inline]
    pub fn get(&self, index: usize) -> Option<&u8> {
        self.memory.get(index)
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: u8) {
        *self.memory.get_mut(index).unwrap() = value;
    }

    pub fn remove(&mut self, address: usize, len: usize) {
        // NOTE: maybe there is no need to reset the memory to zeros
        for i in 0..len {
            self.memory[(address + i)] = 0;
        }
        self.free.push((address, len));

        self.free.sort_unstable();
        let mut new_free = vec![*self.free.first().unwrap()];
        for (address, remaining) in self.free[1..].iter() {
            let (last_address, last_remaining) = new_free.last_mut().unwrap();
            if *address == *last_address + *last_remaining {
                *last_remaining += remaining;
            } else {
                new_free.push((*address, *remaining))
            }
        }
        self.free = new_free;
    }
}

#[test]
fn test_memory() {
    let mut memory = Memory::new();
    assert_eq!(memory.memory, vec![]);
    assert_eq!(memory.free, vec![(0, usize::MAX)]);
    let address = memory.extend(&[1, 1, 1, 1]);
    assert_eq!(address, 0);
    let address = memory.extend(&[2, 2, 2]);
    assert_eq!(address, 4);
    let address = memory.push(3);
    assert_eq!(address, 7);
    assert_eq!(memory.memory, vec![1, 1, 1, 1, 2, 2, 2, 3]);
    assert_eq!(memory.free, vec![(8, usize::MAX - 8)]);
    memory.remove(1, 4);
    assert_eq!(memory.memory, vec![1, 0, 0, 0, 0, 2, 2, 3]);
    assert_eq!(memory.free, vec![(1, 4), (8, usize::MAX - 8)]);
    let address = memory.push(4);
    assert_eq!(address, 1);
    assert_eq!(memory.memory, vec![1, 4, 0, 0, 0, 2, 2, 3]);
    assert_eq!(memory.free, vec![(2, 3), (8, usize::MAX - 8)]);
    let address = memory.extend(&[5]);
    assert_eq!(address, 2);
    assert_eq!(memory.memory, vec![1, 4, 5, 0, 0, 2, 2, 3]);
    assert_eq!(memory.free, vec![(3, 2), (8, usize::MAX - 8)]);
    let address = memory.extend(&[6, 6, 6]);
    assert_eq!(address, 8);
    assert_eq!(memory.memory, vec![1, 4, 5, 0, 0, 2, 2, 3, 6, 6, 6]);
    assert_eq!(memory.free, vec![(3, 2), (11, usize::MAX - 11)]);
    let address = memory.extend(&[7, 7]);
    assert_eq!(address, 3);
    assert_eq!(memory.memory, vec![1, 4, 5, 7, 7, 2, 2, 3, 6, 6, 6]);
    assert_eq!(memory.free, vec![(11, usize::MAX - 11)]);
    memory.remove(4, 1);
    let address = memory.push(8);
    assert_eq!(address, 4);
    assert_eq!(memory.memory, vec![1, 4, 5, 7, 8, 2, 2, 3, 6, 6, 6]);
    assert_eq!(memory.free, vec![(11, usize::MAX - 11)]);
    memory.remove(0, memory.memory.len());
    assert_eq!(memory.memory, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(memory.free, vec![(0, usize::MAX)]);

    let mut memory = Memory::new();
    memory.alloc(5);
    assert_eq!(memory.memory, vec![0, 0, 0, 0, 0]);
    assert_eq!(memory.free, vec![(0, usize::MAX - 5)]);
}
