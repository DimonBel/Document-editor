pub mod room; pub mod document; pub mod latex; pub mod audit;
pub struct Topics;
impl Topics {
    pub fn for_ctx(c: &str, a: &str, e: &str) -> String { format!("{}.{}.{}", c.to_lowercase(), a.to_lowercase(), e.to_lowercase()) }
}
