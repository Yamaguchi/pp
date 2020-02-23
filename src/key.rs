use std::cmp::*;
use std::fmt;
use std::marker::PhantomData;

pub struct PrivateKey<T> {
    pub inner: Vec<u8>,
    pub _phantom: PhantomData<T>,
}

impl<T> fmt::Display for PrivateKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.inner.clone()))
    }
}

impl<T> fmt::Debug for PrivateKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", hex::encode(self.inner.clone()))
    }
}

impl<T> Clone for PrivateKey<T> {
    fn clone(&self) -> Self {
        PrivateKey {
            inner: self.inner.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<T> PrivateKey<T> {
    pub fn new(bytes: &[u8]) -> Self {
        PrivateKey::<T> {
            inner: Vec::from(bytes),
            _phantom: PhantomData,
        }
    }
}
impl<T> Eq for PrivateKey<T> {}

impl<T> PartialEq for PrivateKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T> Ord for PrivateKey<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<T> PartialOrd for PrivateKey<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

pub struct PublicKey<T> {
    pub inner: Vec<u8>,
    pub _phantom: PhantomData<T>,
}

impl<T> fmt::Display for PublicKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.inner.clone()))
    }
}

impl<T> fmt::Debug for PublicKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", hex::encode(self.inner.clone()))
    }
}

impl<T> Clone for PublicKey<T> {
    fn clone(&self) -> Self {
        PublicKey {
            inner: self.inner.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<T> Eq for PublicKey<T> {}

impl<T> PartialEq for PublicKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T> Ord for PublicKey<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<T> PartialOrd for PublicKey<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}
