pub struct SortedVec<T> {
    pub vec: Vec<T>,
}

impl<T: PartialOrd> SortedVec<T> {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }

    pub fn add(&mut self, value: T) {
        let mut left = 0;
        let mut right = self.vec.len();

        while left < right {
            let mid = (left + right) / 2;

            if self.vec[mid] < value {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        self.vec.insert(left, value);
    }

    pub fn first(&self) -> Option<&T> {
        self.vec.first()
    }

    pub fn pop(&mut self) -> Option<T> {
        self.vec.pop()
    }
}
