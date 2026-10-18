#[derive(Clone)]
pub struct AbsolutePath {
    components: Vec<String>,
}

impl AbsolutePath {
    #[must_use]
    pub fn push(&self, component: impl Into<String>) -> Self {
        let mut path = self.clone();
        path.components.push(component.into());
        path
    }
}

impl<'a> FromIterator<&'a str> for AbsolutePath {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self {
            components: iter.into_iter().map(str::to_string).collect::<Vec<_>>(),
        }
    }
}

#[derive(Clone)]
pub enum Path {
    Global { name: String },
    Absolute { components: Vec<String> },
}

impl Path {
    pub fn from_global(path: impl Into<String>) -> Self {
        Self::Global { name: path.into() }
    }

    #[must_use]
    pub fn from_absolute(path: AbsolutePath) -> Self {
        Self::Absolute {
            components: path.components,
        }
    }
}

//impl Path {
//    #[must_use]
//    pub const fn components(&self) -> &[String] {
//        match self {
//            Path::Global { name } => todo!(),
//            Path::Absolute { components } => todo!(),
//        }
//        self.components.as_slice()
//    }
//}
