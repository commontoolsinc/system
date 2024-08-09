use crate::{CommonIfcError, Data};
use common_macros::Lattice;
use std::{
    fmt::{Debug, Display},
    slice::Iter,
    str::FromStr,
};

/// Trait representing a partially ordered lattice.
pub trait Lattice: Display + Debug + Ord + PartialOrd + Eq + PartialEq + Sized {
    /// Return the top-most principal for this lattice.
    fn top() -> Self;
    /// Return the bottom-most principal for this lattice.
    fn bottom() -> Self;
    /// Returns an iterator that iterates over
    /// all variants, from lowest to highest.
    fn iter() -> Iter<'static, Self>;
}

/// Contains the [Confidentiality] and
/// [Integrity] describing the confidentiality
/// and integrity of data `T` associated with [Data].
#[derive(PartialEq, Debug, Clone, Default)]
pub struct Label {
    /// Confidentiality component of [Label].
    pub confidentiality: Confidentiality,
    /// Integrity component of [Label].
    pub integrity: Integrity,
}

impl Label {
    /// Create a [Label] that sets its [Confidentiality]
    /// and [Integrity] to the highest confidentiality
    /// and lowest integrity found in `input`.
    pub fn constrain<'a, T, I>(input: I) -> Self
    where
        T: 'static,
        I: IntoIterator<Item = (&'a String, &'a Data<T>)>,
    {
        let mut max_conf = Confidentiality::bottom();
        let mut min_int = Integrity::top();
        for (_, data) in input {
            let (conf, int) = (&data.label).into();
            max_conf = std::cmp::max(max_conf, conf);
            min_int = std::cmp::min(min_int, int);
        }
        (max_conf, min_int).into()
    }
}

impl From<(Confidentiality, Integrity)> for Label {
    fn from(value: (Confidentiality, Integrity)) -> Self {
        Label {
            confidentiality: value.0,
            integrity: value.1,
        }
    }
}

impl From<Label> for (Confidentiality, Integrity) {
    fn from(value: Label) -> (Confidentiality, Integrity) {
        (value.confidentiality, value.integrity)
    }
}

impl From<&Label> for (Confidentiality, Integrity) {
    fn from(value: &Label) -> (Confidentiality, Integrity) {
        value.to_owned().into()
    }
}

/// Levels of integrity for a [Data], ordered
/// from least to most integrity.
#[derive(Lattice, Default, Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub enum Integrity {
    /// The lowest integrity label.
    #[default]
    LowIntegrity,
    /// The highest integrity label.
    HighIntegrity,
}

impl Display for Integrity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Integrity::*;
        write!(
            f,
            "{}",
            match self {
                HighIntegrity => "HighIntegrity",
                LowIntegrity => "LowIntegrity",
            }
        )
    }
}

impl FromStr for Integrity {
    type Err = CommonIfcError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Integrity::*;
        match s {
            "HighIntegrity" => Ok(HighIntegrity),
            "LowIntegrity" => Ok(LowIntegrity),
            _ => Err(CommonIfcError::Conversion),
        }
    }
}

/// Levels of confidentiality for a [Data], ordered
/// from least to most confidential.
#[derive(Lattice, Default, Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub enum Confidentiality {
    /// The lowest confidentiality label.
    Public,
    /// The highest confidentiality label.
    #[default]
    Private,
}

impl Display for Confidentiality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Confidentiality::*;
        write!(
            f,
            "{}",
            match self {
                Private => "Private",
                Public => "Public",
            }
        )
    }
}

impl FromStr for Confidentiality {
    type Err = CommonIfcError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Confidentiality::*;
        match s {
            "Private" => Ok(Private),
            "Public" => Ok(Public),
            _ => Err(CommonIfcError::Conversion),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use Confidentiality::*;
    use Integrity::*;

    #[test]
    fn it_implements_ordered_lattice_trait() {
        check_top_gt_bottom::<Confidentiality>();
        check_top_gt_bottom::<Integrity>();
        check_iter_order::<Confidentiality>();
        check_iter_order::<Integrity>();

        fn check_top_gt_bottom<T: Lattice>() {
            assert!(T::top() > T::bottom());
        }

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

    #[test]
    fn it_constrains_from_input() {
        let private_high = ("vh".into(), Data::from(("data", Private, HighIntegrity)));
        let private_low = ("vl".into(), Data::from(("data", Private, LowIntegrity)));
        let public_high = ("bh".into(), Data::from(("data", Public, HighIntegrity)));
        let public_low = ("bl".into(), Data::from(("data", Public, LowIntegrity)));

        fn input(seq: [&(String, Data<&'static str>); 2]) -> BTreeMap<String, Data<&'static str>> {
            BTreeMap::from(seq.map(|i| i.to_owned()))
        }

        assert_eq!(
            Label::constrain(&input([&private_high, &public_low])),
            (Private, LowIntegrity).into(),
        );
        assert_eq!(
            Label::constrain(&input([&private_low, &public_high])),
            (Private, LowIntegrity).into(),
        );
        assert_eq!(
            Label::constrain(&input([&public_low, &public_high])),
            (Public, LowIntegrity).into(),
        );
        assert_eq!(
            Label::constrain(&input([&private_low, &private_high])),
            (Private, LowIntegrity).into(),
        );
        assert_eq!(
            Label::constrain(&input([&private_high, &private_high])),
            (Private, HighIntegrity).into(),
        );
    }
}
