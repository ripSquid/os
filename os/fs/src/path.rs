use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[derive(Clone)]
pub struct Path(pub(crate) String);
impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
    }
}

impl Path {
    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.0.split("/")
    }
    pub fn parent(&self) -> Option<Self> {
        let clean = self.clone().clean();
        let segments: Vec<_> = clean.components().collect();
        if segments.len() < 1 {
            return None;
        }
        Some(Self::from_segments(&segments[..segments.len() - 1]))
    }
    pub fn pop(&mut self) -> Option<Path> {
        let clean = self.clone().clean();
        let segments: Vec<_> = clean.components().collect();
        if segments.len() < 1 {
            return None;
        }
        *self = Self::from_segments(&segments[..segments.len() - 1]);
        Some(Self::from(*segments.last()?))
    }
    pub fn from_segments(items: &[&str]) -> Self {
        Self(items.iter().fold(String::new(), |mut x, i| {
            x.push('/');
            x.push_str(i);
            x
        }))
    }
    pub fn new() -> Self {
        Self(String::new())
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
    pub fn append<A: AppendsPath>(mut self, path: &A) -> Self {
        self.0.push_str("/");
        self.0.push_str(path.to_str());
        self
    }
    pub fn clean(self) -> Self {
        let mut new = Vec::new();
        let sections = self.0.split('/');
        for section in sections {
            match section {
                "" => continue,
                "." => continue,
                ".." => {
                    new.pop();
                    continue;
                }
                _ => (),
            }
            new.push(section);
        }
        let mut finale = String::new();
        for part in new.into_iter() {
            finale.push('/');
            finale.push_str(part);
        }
        Self(finale)
    }
    pub fn add_extension(mut self, extension: &str) -> Self {
        self.0.push('.');
        self.0.push_str(extension);
        self
    }
}
pub trait AppendsPath {
    fn to_str(&self) -> &str;
}
impl<T: AsRef<str>> AppendsPath for T {
    fn to_str(&self) -> &str {
        self.as_ref()
    }
}
impl AppendsPath for Path {
    fn to_str(&self) -> &str {
        self.as_ref().as_str()
    }
}
impl From<String> for Path {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for Path {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
