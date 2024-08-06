use common_macros::NewType;

#[derive(NewType, PartialEq, Debug)]
struct Foo(String);

#[test]
fn it_adds_methods_and_from_traits() {
    let string: String = "foo".into();
    let foo = Foo::from(string.clone());

    assert_eq!(foo, Foo::from(string.clone()));
    assert_eq!(string, String::from(foo.clone()));
    assert_eq!(foo.into_inner(), string);
}

#[test]
fn it_adds_deref_traits() {
    use std::ops::{Deref, DerefMut};
    let string: String = "foo".into();
    let foo = Foo::from(string.clone());
    assert_eq!(&string, foo.deref());

    let mut foo = Foo::from(string.clone());
    let mut_foo = foo.deref_mut();
    mut_foo.push_str("bar");
    assert_eq!(foo.deref(), &String::from("foobar"));
}

#[test]
fn it_can_use_pub_types() {
    #[derive(NewType)]
    struct PubType(pub String);
}

#[test]
fn it_handles_generics() {
    #[derive(NewType)]
    struct Buffer<T>(Vec<T>);

    let vec: Vec<u8> = vec![0];
    let buffer = Buffer::from(vec.clone());
    assert_eq!(vec, Vec::from(buffer));
}
