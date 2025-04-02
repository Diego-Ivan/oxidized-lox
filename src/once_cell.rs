use std::cell::RefCell;
use std::ops::Deref;

pub struct OnceCell<T> {
    cell: RefCell<Option<T>>,
}

impl<T> OnceCell<T> {
    pub fn new_empty() -> Self {
        Self {
            cell: RefCell::new(None),
        }
    }

    pub fn new(inner: T) -> Self {
        Self {
            cell: RefCell::new(Some(inner)),
        }
    }

    pub fn take(&self) -> T {
        self.cell.take().expect("Called take on a empty OnceCell")
    }

    pub fn replace(&self, new: T) {
        self.cell.replace(Some(new));
    }
}

impl<T> Deref for OnceCell<T> {
    type Target = RefCell<Option<T>>;

    fn deref(&self) -> &Self::Target {
        &self.cell
    }
}
