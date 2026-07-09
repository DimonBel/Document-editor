use std::hash::Hash;
pub trait ValueObject: Eq + Hash + Clone {
    fn get_equality_components(&self) -> Vec<Box<dyn std::any::Any>>;
}
