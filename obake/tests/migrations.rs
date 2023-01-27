#![allow(unused)]

#[obake::versioned]
#[obake(version(1))]
#[obake(version(2))]
#[obake(version(3))]
#[derive(Default)]
struct Foo {
    field_0: u32,
    #[obake(cfg(2))]
    field_1: String,
    #[obake(cfg(1))]
    #[obake(cfg(3))]
    field_2: i64,
}

impl From<Foo![1]> for Foo![2] {
    fn from(from: Foo![1]) -> Self {
        Self {
            field_0: from.field_0,
            field_1: "default".to_owned(),
        }
    }
}

impl From<Foo![2]> for Foo![3] {
    fn from(from: Foo![2]) -> Self {
        Self {
            field_0: from.field_0,
            field_2: 42,
        }
    }
}

#[obake::versioned]
#[obake(version(1))]
#[obake(version(2))]
#[obake(version(3))]
#[derive(Default)]
struct Bar {
    #[obake(inherit)]
    #[obake(cfg(2..))]
    field_0: Foo,
}

impl From<Bar![1]> for Bar![2] {
    fn from(from: Bar![1]) -> Self {
        Default::default()
    }
}

impl From<Bar![2]> for Bar![3] {
    fn from(from: Bar![2]) -> Self {
        Self {
            field_0: from.field_0.into(),
        }
    }
}

#[obake::versioned]
#[obake(version(1))]
#[obake(version(2))]
#[obake(version(3))]
enum Baz {
    #[obake(cfg(..3))]
    X(String),
    #[obake(cfg(2..))]
    Y {
        #[obake(inherit)]
        #[obake(cfg(2..))]
        foo: Foo,
        #[obake(inherit)]
        #[obake(cfg(2..))]
        bar: Bar,
    },
}

impl From<Baz![1]> for Baz![2] {
    fn from(from: Baz![1]) -> Self {
        type Baz = Baz![1];
        match from {
            Baz::X(x) => Self::X(x),
        }
    }
}

impl From<Baz![2]> for Baz![3] {
    fn from(from: Baz![2]) -> Self {
        type Baz = Baz![2];
        match from {
            Baz::X(_) => Self::Y {
                foo: Default::default(),
                bar: Default::default(),
            },
            Baz::Y { foo, bar } => Self::Y {
                foo: foo.into(),
                bar: bar.into(),
            },
        }
    }
}
