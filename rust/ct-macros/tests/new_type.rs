use ct_macros::NewType;

#[test]
fn it_derives_from_inner() {
    #[derive(NewType)]
    struct Foo(pub String);

    let s = String::from("foo");
    let f = Foo::from(s.clone());
    assert_eq!(f.0, s);
}

#[test]
fn it_derives_into_inner() {
    #[derive(NewType)]
    struct Foo(pub String);

    let s = String::from("foo");
    let f = Foo(s.clone());
    let out: String = f.into();
    assert_eq!(out, s);
}

#[test]
fn it_derives_deref() {
    use std::ops::{Deref as DerefTrait, DerefMut as DerefMutTrait};
    #[derive(Clone, NewType)]
    struct Foo(pub String);

    let s = String::from("foo");
    let f = Foo(s.clone());
    assert_eq!(&s, f.deref());

    let mut f = f.clone();
    let mut_foo = f.deref_mut();
    mut_foo.push_str("bar");
    assert_eq!(f.deref(), &String::from("foobar"));
}

#[test]
fn it_derives_constructor() {
    #[derive(Clone, NewType)]
    struct Foo(pub String);

    let s = String::from("foo");
    let f = Foo::new(s.clone());
    assert_eq!(f.0, s);
}

#[test]
fn it_derives_inner() {
    #[derive(Clone, NewType)]
    struct Foo(pub String);

    let s = String::from("foo");
    let mut f = Foo(s.clone());
    assert_eq!(f.inner(), &s);

    {
        let inner = f.inner_mut();
        inner.push_str("bar");
    }

    assert_eq!(f.into_inner(), String::from("foobar"));
}

#[test]
fn it_handles_generics() {
    #[derive(NewType)]
    struct Buffer<T>(Vec<T>);

    let vec: Vec<u8> = vec![0];
    let buffer = Buffer::from(vec.clone());
    assert_eq!(vec, Vec::from(buffer));
}

#[test]
fn it_only_includes_traits_helper() {
    #[derive(NewType)]
    #[new_type(only(From))]
    struct Foo(pub String);

    // This will fail if already defined
    impl From<Foo> for String {
        fn from(value: Foo) -> Self {
            value.0
        }
    }

    let s = String::from("foo");
    let f = Foo::from(s.clone());
    assert_eq!(f.0, s);
}

#[test]
fn it_skips_includes_traits_helper() {
    #[derive(NewType)]
    #[new_type(skip(From))]
    struct Foo(pub String);

    // This will fail if already defined
    impl From<String> for Foo {
        fn from(value: String) -> Self {
            Foo(value)
        }
    }

    let s = String::from("foo");
    let f = Foo::from(s.clone());
    assert_eq!(String::from(f), s);
}

#[test]
fn it_prioritizes_only_over_skip() {
    #[derive(NewType)]
    #[new_type(only(Constructor), skip(Constructor))]
    struct Foo(pub String);

    // This will fail if already defined
    impl From<String> for Foo {
        fn from(value: String) -> Self {
            Foo(value)
        }
    }

    let s = String::from("foo");
    let f = Foo::new(s.clone());
    assert_eq!(f.0, s);
}
