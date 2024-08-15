use crate::ifc::{HasLabelType, IfcLabel, LabelType};
use common_macros::Lattice;
use std::fmt::Debug;

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

impl IfcLabel for Label {
    type Confidentiality = Confidentiality;
    type Integrity = Integrity;
    fn confidentiality(&self) -> &Self::Confidentiality {
        &self.confidentiality
    }
    fn integrity(&self) -> &Self::Integrity {
        &self.integrity
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
#[derive(
    strum::Display,
    strum::EnumString,
    Lattice,
    Copy,
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
    #[strum(serialize = "LowIntegrity")]
    #[default]
    Low,
    /// The highest integrity label.
    #[strum(serialize = "HighIntegrity")]
    High,
}

impl HasLabelType for Integrity {
    fn label_type() -> LabelType {
        LabelType::Integrity
    }
}

/// Levels of confidentiality for a [Data], ordered
/// from least to most confidential.
#[derive(
    strum::Display,
    strum::EnumString,
    Lattice,
    Copy,
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
    #[strum(serialize = "LowConfidentiality")]
    Low,
    /// The highest confidentiality label.
    #[strum(serialize = "HighConfidentiality")]
    #[default]
    High,
}

impl HasLabelType for Confidentiality {
    fn label_type() -> LabelType {
        LabelType::Confidentiality
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ifc::Lattice;
    use std::collections::BTreeMap;

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
        let private_high = ("vh".into(), (Confidentiality::High, Integrity::High).into());
        let private_low = ("vl".into(), (Confidentiality::High, Integrity::Low).into());
        let public_high = ("bh".into(), (Confidentiality::Low, Integrity::High).into());
        let public_low = ("bl".into(), (Confidentiality::Low, Integrity::Low).into());

        fn input(seq: [&(String, Label); 2]) -> BTreeMap<String, Label> {
            BTreeMap::from(seq.map(|i| i.to_owned()))
        }

        assert_eq!(
            Label::constrain(&input([&private_high, &public_low])),
            (Confidentiality::High, Integrity::Low).into(),
        );
        assert_eq!(
            Label::constrain(&input([&private_low, &public_high])),
            (Confidentiality::High, Integrity::Low).into(),
        );
        assert_eq!(
            Label::constrain(&input([&public_low, &public_high])),
            (Confidentiality::Low, Integrity::Low).into(),
        );
        assert_eq!(
            Label::constrain(&input([&private_low, &private_high])),
            (Confidentiality::High, Integrity::Low).into(),
        );
        assert_eq!(
            Label::constrain(&input([&private_high, &private_high])),
            (Confidentiality::High, Integrity::High).into(),
        );
    }
}
