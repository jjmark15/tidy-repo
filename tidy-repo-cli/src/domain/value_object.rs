pub trait ValueObject<T> {
    fn value(&self) -> &T;
}
