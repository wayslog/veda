pub struct MinHeap<T> {
    data: Vec<T>,
}

impl<T: Clone> MinHeap<T> {
    pub(crate) fn list(&self) -> Vec<T> {
        self.data.clone()
    }
}

impl<T: Ord> MinHeap<T> {
    pub fn new() -> MinHeap<T> {
        MinHeap { data: Vec::new() }
    }

    pub fn replace_with<F>(&mut self, f: F)
    where
        F: Fn(&mut T),
    {
        self.data.iter_mut().for_each(|x| f(x));

        self.init();
    }

    pub fn replace_with_any<F>(&mut self, f: F) -> bool
    where
        F: Fn(&mut T) -> bool,
    {
        let mut i = 0;
        let replaced = self
            .data
            .iter_mut()
            .enumerate()
            .map(|(idx, x)| {
                i = idx;
                f(x)
            })
            .any(|x| x);

        if replaced {
            self.fix(i);
        }
        replaced
    }

    pub fn peek(&self) -> Option<&T> {
        self.data.first()
    }

    pub fn from_vec(data: Vec<T>) -> MinHeap<T> {
        let mut heap = MinHeap { data };
        heap.init();
        heap
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn push(&mut self, t: T) {
        self.data.push(t);
        self.up(self.len() - 1);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.data.is_empty() {
            return None;
        }
        let n = self.len() - 1;
        self.swap(0, n);
        self.down(0, n);
        self.data.pop()
    }

    pub fn remove(&mut self, i: usize) -> Option<T> {
        let n = self.len() - 1;
        if n != i {
            self.swap(i, n);
            if !self.down(i, n) {
                self.up(i);
            }
        }
        self.pop()
    }

    fn init(&mut self) {
        let n = self.len();
        for i in (0..=(n / 2 - 1)).into_iter().rev() {
            self.down(i, n);
        }
    }

    fn swap(&mut self, i: usize, j: usize) {
        self.data.swap(i, j);
    }

    pub fn fix(&mut self, i: usize) {
        if !self.down(i, self.len()) {
            self.up(i);
        }
    }

    fn down(&mut self, i0: usize, n: usize) -> bool {
        let mut i = i0;
        loop {
            let j1 = i.wrapping_mul(2).wrapping_add(1);
            if j1 >= n || j1 < i {
                // j1 < i means overflow
                break;
            }
            let mut j = j1;
            let j2 = j1 + 1;
            if j2 < n && self.data[j2] < self.data[j1] {
                j = j2;
            }
            if !(self.data[j] < self.data[i]) {
                break;
            }
            self.data.swap(i, j);
            i = j
        }
        i > i0
    }

    fn up(&mut self, mut j: usize) {
        loop {
            if j == 0 {
                return;
            }
            let i = (j - 1) / 2;
            if i == j || !(self.data[j] < self.data[i]) {
                break;
            }
            self.data.swap(i, j);
            j = i;
        }
    }
}

#[test]
fn test_heap() {
    let data = vec![3, 2, 1, 0];
    let heap = MinHeap::from_vec(data);
    for i in heap.data {
        println!("{}", i);
    }
}

#[test]
fn test_heap_push_pop() {
    let data = vec![3, 2, 1, 0];
    let mut heap = MinHeap::new();
    for data in data {
        println!("{data}");
        heap.push(data);
    }

    while let Some(i) = heap.pop() {
        println!("pop: {}", i);
    }
}

#[test]
fn test_heap_replace_with() {
    let data = vec![3, 2, 1, 0];
    let mut heap = MinHeap::from_vec(data);

    heap.replace_with_any(|x| {
        if *x == 1 {
            *x *= 10;
            true
        } else {
            false
        }
    });
    for i in heap.data {
        println!("{}", i);
    }
}
