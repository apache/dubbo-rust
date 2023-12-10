pub trait Param<T> {
    fn param(&self) -> T;
}

impl<T: ToOwned> Param<T::Owned> for T {
    fn param(&self) -> T::Owned {
        self.to_owned()
    }
}
