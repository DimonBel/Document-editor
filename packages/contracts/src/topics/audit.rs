pub fn recorded(ctx: &str) -> String { format!("{}.audit.recorded", ctx) }
pub const DEAD_LETTER: &str = "audit.recorded.dlq";
pub const POISON: &str = "audit.recorded.poison";
