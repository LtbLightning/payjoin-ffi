use std::sync::Arc;

pub mod v1;
pub mod v2;

struct Context(Arc<payjoin::send::Context>);
impl From<payjoin::send::Context> for Context {
    fn from(value: payjoin::send::Context) -> Self {
        Self(Arc::new(value))
    }
}
