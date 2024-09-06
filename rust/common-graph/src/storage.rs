use super::{CommonGraphError, PortType, Result};
use crate::utils::is_full;
use common_macros::NewType;

#[cfg(doc)]
use crate::Graph;

/// A set of input or output ports in [`GraphData<V>`].
pub type GraphDataInner<'a, V> = Vec<(&'a str, Option<V>)>;

/// Contains a tuple of input and output ports,
/// each containing a tuple of port name and its
/// current value, which may or may not be set.
#[derive(NewType, Clone)]
#[new_type(only(Inner, Into, From))]
pub struct GraphData<'a, V>(Vec<(GraphDataInner<'a, V>, GraphDataInner<'a, V>)>);

impl<'a, V> GraphData<'a, V> {
    /// Convert this [`GraphData<V>`] into its owned
    /// counterpart, [`OwnedGraphData<V>`].
    pub fn into_owned(self) -> OwnedGraphData<V> {
        self.0
            .into_iter()
            .map(|(ins, outs)| {
                (
                    ins.into_iter().map(|(k, v)| (String::from(k), v)).collect(),
                    outs.into_iter()
                        .map(|(k, v)| (String::from(k), v))
                        .collect(),
                )
            })
            .collect::<Vec<_>>()
            .into()
    }
}

/// A set of input or output ports in [`OwnedGraphData<V>`].
pub type OwnedGraphDataInner<V> = Vec<(String, Option<V>)>;

/// An "owned" version of [`GraphData`] where the keys
/// are [`String`]s rather than references to a [`Graph`].
#[derive(NewType, Clone)]
#[new_type(only(Inner, Into, From))]
pub struct OwnedGraphData<V>(Vec<(OwnedGraphDataInner<V>, OwnedGraphDataInner<V>)>);

type PortStoreMut<'store, 'a, V> = (
    &'store Vec<(&'a str, Option<V>)>,
    Vec<(&'a str, &'store mut Option<V>)>,
);

/// Storage for port data while traversing a [`Graph`].
/// Used to collect data during traversal, and generate
/// completed graph traversal representations.
///
/// Must be initialized up front with all port names
/// of nodes, as node state cares about whether
/// their inputs are "full" or not.
#[derive(NewType)]
#[new_type(only(Into))]
pub struct GraphStorage<'a, V>(GraphData<'a, V>);

impl<'a, V> GraphStorage<'a, V> {
    /// Initializes storage for all nodes with the provided keys.
    pub(crate) fn from_iter<I1, I2, I3>(iter: I1) -> Self
    where
        I1: IntoIterator<Item = (I2, I3)>,
        I2: IntoIterator<Item = &'a str>,
        I3: IntoIterator<Item = &'a str>,
    {
        let mut store = vec![];
        for (inputs, outputs) in iter {
            let inputs: Vec<_> = inputs.into_iter().map(|key| (key, None)).collect();
            let outputs: Vec<_> = outputs.into_iter().map(|key| (key, None)).collect();
            store.push((inputs, outputs));
        }
        GraphStorage(store.into())
    }

    /// Sets the value of node `index`'s `port_type` port with name
    /// `port_name` to `value`.
    pub(crate) fn set(
        &mut self,
        index: usize,
        port_name: &str,
        value: V,
        port_type: PortType,
    ) -> Result<()> {
        self.check_index(index)?;
        let (ref mut inputs, ref mut outputs) = &mut self.0.inner_mut()[index];
        let ports = match port_type {
            PortType::Input => inputs,
            PortType::Output => outputs,
        };

        let (_, port_value) = ports
            .iter_mut()
            .find(|(key, _)| *key == port_name)
            .ok_or_else(|| {
                CommonGraphError::Unexpected("Attempted to write to unknown port.".into())
            })?;
        *port_value = Some(value);
        Ok(())
    }

    /// Returns the value of node `index`'s `port_type` port with name
    /// `port_name`.
    pub(crate) fn get(
        &self,
        index: usize,
        port_name: &str,
        port_type: PortType,
    ) -> Result<Option<&V>> {
        self.check_index(index)?;
        let (ref inputs, ref outputs) = &self.0.inner()[index];
        let ports = match port_type {
            PortType::Input => inputs,
            PortType::Output => outputs,
        };

        let (_, port_value) = ports
            .iter()
            .find(|(key, _)| *key == port_name)
            .ok_or_else(|| {
                CommonGraphError::Unexpected("Attempted to get an unknown port.".into())
            })?;
        Ok(port_value.as_ref())
    }

    /// Returns both inputs and outputs for `index`, with
    /// the outputs' values being mutable.
    pub(crate) fn get_io_mut<'store>(
        &'store mut self,
        index: usize,
    ) -> Result<PortStoreMut<'store, 'a, V>> {
        self.check_index(index)?;
        let (ref inputs, ref mut outputs) = &mut self.0.inner_mut()[index];
        // We create a new Vec here because we want to
        // return mutable references to underlying values,
        // but don't want to have the underlying collection or keys
        // to be mutable. Maybe a better way?
        let outputs = outputs.iter_mut().map(|(k, v)| (*k, v)).collect::<Vec<_>>();
        Ok((inputs, outputs))
    }

    /// Whether node at `index`'s `port_type` ports are all
    /// `Some`.
    pub(crate) fn is_full(&self, index: usize, port_type: PortType) -> bool {
        let Some(ports) = self.0.inner().get(index) else {
            return false;
        };
        match port_type {
            PortType::Input => is_full(&ports.0),
            PortType::Output => is_full(&ports.1),
        }
    }

    fn check_index(&self, index: usize) -> Result<()> {
        match index >= self.0.inner().len() {
            true => Err(CommonGraphError::Unexpected(
                "GraphStorage request out of range.".into(),
            )),
            false => Ok(()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_stores_data() -> Result<()> {
        let mut storage = GraphStorage::<usize>::from_iter([
            (vec!["A1", "A2"], vec!["A3", "A4"]),
            (vec!["B1"], vec!["B2"]),
        ]);

        assert_eq!(storage.get(0, "A1", PortType::Input)?, None);
        storage.set(0, "A1", 1000, PortType::Input)?;
        assert_eq!(storage.get(0, "A1", PortType::Input)?, Some(&1000));
        assert!(!storage.is_full(0, PortType::Input));
        storage.set(0, "A2", 1100, PortType::Input)?;
        assert!(storage.is_full(0, PortType::Input));
        assert!(!storage.is_full(0, PortType::Output));

        storage.set(1, "B2", 2000, PortType::Output)?;
        assert_eq!(storage.get(1, "B2", PortType::Output)?, Some(&2000));
        assert!(storage.is_full(1, PortType::Output));

        Ok(())
    }

    #[test]
    fn it_throws_when_out_of_range() -> Result<()> {
        let mut storage = GraphStorage::<()>::from_iter([
            (vec!["A1", "A2"], vec!["A3", "A4"]),
            (vec!["B1"], vec!["B2"]),
        ]);

        // Node out of range
        assert!(storage.set(3, "A1", (), PortType::Input).is_err());
        assert!(storage.get(3, "A1", PortType::Input).is_err());
        // Non-existant ports
        assert!(storage.set(0, "A3", (), PortType::Input).is_err());
        assert!(storage.set(0, "X", (), PortType::Input).is_err());
        assert!(storage.get(0, "A1", PortType::Output).is_err());
        Ok(())
    }
}
