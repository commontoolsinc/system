use common_ifc::Lattice;

#[derive(Lattice, PartialOrd, Ord, PartialEq, Eq, Debug)]
enum Seasons {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl std::fmt::Display for Seasons {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Seasons::Spring => "Spring",
                Seasons::Summer => "Summer",
                Seasons::Autumn => "Autumn",
                Seasons::Winter => "Winter",
            },
        )
    }
}

#[test]
fn it_adds_iter() {
    let seasons: Vec<&Seasons> = Seasons::iter().collect();
    assert_eq!(
        seasons,
        vec![
            &Seasons::Spring,
            &Seasons::Summer,
            &Seasons::Autumn,
            &Seasons::Winter,
        ]
    );
}

#[test]
fn it_orders_iter() {
    check_iter_order::<Seasons>();

    fn check_iter_order<T: 'static + Lattice>() {
        let mut previous = None;
        for level in T::iter() {
            if let Some(previous) = previous {
                assert!(previous < level);
            } else {
                assert_eq!(level, &T::bottom())
            }
            previous = Some(level);
        }
        assert_eq!(previous.unwrap(), &T::top());
    }
}
