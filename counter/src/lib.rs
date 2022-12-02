pub struct Counter {
    val: i8,
}

impl Counter {
    pub fn get_num(&self) -> i8 {
        return self.val;
    }

    pub fn increment(&mut self) {
        self.val += 1;
    }

    pub fn decrement(&mut self) {
        self.val -= 1;
    }

    pub fn reset(&mut self) {
        self.val = 0;
    }
}

//Test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increment() {
        // set the init value for counter = 0
        let mut init = Counter { val: 0 };
        init.increment();
        assert_eq!(1, init.get_num())
    }

    #[test]
    fn decrement() {
        //set the init value for counter = 0
        let mut init = Counter { val: 0 };
        init.decrement();
        assert_eq!(-1, init.get_num())
    }

    #[test]
    #[should_panic]
    fn panics_on_overflow() {
        let mut init = Counter { val: 127 };
        init.increment()
    }

    #[test]
    #[should_panic]
    fn panics_on_underflow() {
        let mut init = Counter { val: -128 };
        init.decrement()
    }
}