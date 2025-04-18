use std::fmt::Display;

/// Types that can generate their own keys for persistent storage
#[uniffi::export]
pub trait Value: serde::Serialize + serde::de::DeserializeOwned + Sized + Clone {
    type Key: AsRef<[u8]> + Clone + Display;

    /// Unique identifier for this persisted value
    fn key(&self) -> Self::Key;
}

/// Implemented types that should be persisted by the application.
#[uniffi::export]
pub trait Persister<V: Value> {
    type Token: From<V>;
    type Error: std::error::Error + Send + Sync + 'static;

    fn save(&mut self, value: V) -> Result<Self::Token, Self::Error>;
    fn load(&self, token: Self::Token) -> Result<V, Self::Error>;
}

/// A key type that stores the value itself for no-op persistence
#[derive(Debug, Clone, serde::Serialize)]
pub struct NoopToken<V: Value>(V);

impl<V: Value> AsRef<[u8]> for NoopToken<V> {
    fn as_ref(&self) -> &[u8] {
        // Since this is a no-op implementation, we can return an empty slice
        // as we never actually need to use the bytes
        &[]
    }
}

impl<'de, V: Value> serde::Deserialize<'de> for NoopToken<V> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(NoopToken(V::deserialize(deserializer)?))
    }
}

impl<V: Value> Value for NoopToken<V> {
    type Key = V::Key;

    fn key(&self) -> Self::Key { self.0.key() }
}

/// A persister that does nothing but store values in memory
#[derive(Debug, Clone)]
pub struct NoopPersister;

impl<V: Value> From<V> for NoopToken<V> {
    fn from(value: V) -> Self { NoopToken(value) }
}
impl<V: Value> Persister<V> for NoopPersister {
    type Token = NoopToken<V>;
    type Error = std::convert::Infallible;

    fn save(&mut self, value: V) -> Result<Self::Token, Self::Error> { Ok(NoopToken(value)) }

    fn load(&self, token: Self::Token) -> Result<V, Self::Error> { Ok(token.0) }
}