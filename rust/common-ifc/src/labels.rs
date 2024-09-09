use common_macros::Lattice;
use std::{
    fmt::{Debug, Display},
    slice::Iter,
};

/// Enum representing either the [`Confidentiality`] or [`Integrity`] lattices.
#[derive(Debug, PartialEq)]
pub enum LabelType {
    /// Represents [`Confidentiality`].
    Confidentiality,
    /// Represents [`Integrity`].
    Integrity,
}

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

/// Contains the [`Confidentiality`] and
/// [`Integrity`] describing the confidentiality
/// and integrity of data `T` associated with data.
#[derive(PartialEq, Debug, Clone, Default)]
pub struct Label {
    /// Confidentiality component of [`Label`].
    pub confidentiality: Confidentiality,
    /// Integrity component of [`Label`].
    pub integrity: Integrity,
}

impl Label {
    /// Create a [`Label`] that sets its [`Confidentiality`]
    /// and [`Integrity`] to the highest confidentiality
    /// found in `input`, and the lowest integrity.
    pub fn constrain<'a, I>(input: I) -> Self
    where
        I: IntoIterator<Item = &'a Label>,
    {
        let mut max_conf = Confidentiality::bottom();
        for label in input {
            let (conf, _) = label.into();
            max_conf = std::cmp::max(max_conf, conf);
        }
        (max_conf, Integrity::bottom()).into()
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

/// Levels of integrity for data, ordered
/// from least to most integrity.
#[derive(
    strum::Display,
    strum::EnumString,
    Lattice,
    Default,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Clone,
    Debug,
)]
pub enum Integrity {
    /// The lowest integrity label.
    #[default]
    #[strum(to_string = "LowIntegrity")]
    Low,
    /// The highest integrity label.
    #[strum(to_string = "HighIntegrity")]
    High,
}

/// Levels of confidentiality for data, ordered
/// from least to most confidential.
#[derive(
    strum::Display,
    strum::EnumString,
    Lattice,
    Default,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Clone,
    Debug,
)]
pub enum Confidentiality {
    /// The lowest confidentiality label.
    #[strum(to_string = "Public")]
    Public,
    /// The highest confidentiality label.
    #[default]
    #[strum(to_string = "Private")]
    Private,
}

#[cfg(feature = "render")]
impl common_graph::RenderableValue for Label {
    fn render_value(&self) -> String {
        self.confidentiality.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let private_high = (Private, High).into();
        let private_low = (Private, Low).into();
        let public_high = (Public, High).into();
        let public_low = (Public, Low).into();

        assert_eq!(
            Label::constrain([&private_high, &public_low]),
            (Private, Low).into(),
        );
        assert_eq!(
            Label::constrain([&private_low, &public_high]),
            (Private, Low).into(),
        );
        assert_eq!(
            Label::constrain([&public_low, &public_high]),
            (Public, Low).into(),
        );
        assert_eq!(
            Label::constrain([&private_low, &private_high]),
            (Private, Low).into(),
        );
        assert_eq!(
            Label::constrain([&private_high, &private_high]),
            (Private, Low).into(),
        );
    }
}
