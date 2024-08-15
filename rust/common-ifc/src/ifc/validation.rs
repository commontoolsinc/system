use common_macros::NewType;

use crate::{
    error::ErrorSource,
    graph::{PortGraph, PortGraphEdge, PortGraphNode, PortType},
    ifc::{IfcGraph, IfcLabel, IfcNode, IfcPolicy},
    CommonIfcError, Result,
};
use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
struct NodeLabels<L: IfcLabel> {
    pub inputs: HashMap<String, L>,
    pub outputs: HashMap<String, L>,
}

impl<L> NodeLabels<L>
where
    L: IfcLabel,
{
    pub fn new() -> Self {
        NodeLabels {
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        }
    }
}

#[derive(NewType, Debug)]
#[new_type(only(Deref, From))]
struct LabelStore<L: IfcLabel>(HashMap<String, NodeLabels<L>>);

const NODE_NOT_FOUND: &str = "Node not found in graph.";
const NO_NODE_LABELS: &str = "Expected node labels to be found.";
const NO_PORT_LABELS: &str = "Expected port labels to be found.";
const EXPECTED_CONTEXT: &str = "Expected node to have a context.";

/// Validate graph input against this policy, given a [Context].
pub fn validate_graph<'a, I, P, G>(graph: &G, policy: &P, input: I) -> Result<()>
where
    I: IntoIterator<Item = &'a (String, <P as IfcPolicy>::Label)>,
    P: IfcPolicy,
    G: IfcGraph<P::Context>,
    <P as IfcPolicy>::Label: 'a,
    <G as PortGraph>::Node: IfcNode<<P as IfcPolicy>::Context>,
{
    graph.check_integrity()?;

    let mut label_store = LabelStore::<P::Label>::from(HashMap::new());
    let Some(root_node) = graph.get_root() else {
        return Err(NODE_NOT_FOUND.into());
    };

    // Seed root node with inputs provided
    {
        let mut node_labels = NodeLabels::<P::Label>::new();
        for (port_name, label) in input.into_iter() {
            if !root_node
                .outputs()
                .any(|out| out.to_string().as_str() == port_name.as_str())
            {
                return Err(format!("Root missing output port {}.", port_name).into());
            }
            node_labels
                .outputs
                .insert(port_name.to_owned(), label.clone());
        }
        label_store.insert(root_node.id().to_string(), node_labels);
    }

    let mut node_queue = VecDeque::from([root_node]);

    while let Some(node) = node_queue.pop_front() {
        debug!("Processing node {}", node.id());

        if !node.is_root() {
            // If node's inputs are not all labeled, skip and put back in the queue.
            if !are_all_ports_labeled::<G, P::Label>(&label_store, node, PortType::Input)? {
                debug!("Skipping node {}, incomplete inputs", node.id());
                node_queue.push_back(node);
                continue;
            }
            // It's possible for multiple branches to queue up the same node.
            // Skip if the node has all of its outputs labeled.
            if are_all_ports_labeled::<G, P::Label>(&label_store, node, PortType::Output)? {
                debug!("Skipping node {}, complete outputs", node.id());
                continue;
            }

            let node_id = node.id().to_string();

            // Propagate all input labels to output labels
            let node_labels = label_store.get_mut(&node_id).ok_or(NO_NODE_LABELS)?;

            let out_label = P::Label::constrain(node_labels.inputs.iter());
            for output in node.outputs() {
                if node_labels
                    .outputs
                    .insert(output.to_string(), out_label.clone())
                    .is_some()
                {
                    warn!("Output label already set??");
                }
            }
        }

        // Propagate labels from outgoing connections
        for edge in graph.get_connections(node, Some(PortType::Output)) {
            let (node_id, node_out) = edge.source();
            let (target_id, target_in) = edge.target();
            let target_node = graph.get_node(target_id).ok_or(NODE_NOT_FOUND)?;
            let node_id = node_id.to_string();
            let target_id = target_id.to_string();
            let node_labels = label_store.get(&node_id).ok_or(NO_NODE_LABELS)?;

            // Validate
            debug!(
                "Validating {}#{} -> {}#{}",
                node_id, node_out, target_id, target_in
            );
            let label = {
                let label = node_labels.outputs.get(node_out).ok_or(NO_PORT_LABELS)?;

                if target_node.is_root() {
                    debug!("Skipping validation for cyclic root input");
                } else {
                    // Validate here. Check the error and provide additional
                    // context if possible.
                    let target_ctx = target_node.context().ok_or(EXPECTED_CONTEXT)?;
                    match policy.check_context(label, target_ctx) {
                        Ok(_) => Ok(()),
                        Err(CommonIfcError::InvalidContext {
                            label_type,
                            details,
                        }) => {
                            let source = ErrorSource {
                                source: node_id,
                                source_port: node_out.into(),
                                target: target_id.clone(),
                                target_port: target_in.into(),
                            };
                            Err(CommonIfcError::ValidationError {
                                error_source: Box::from(source),
                                label_type: label_type.to_string(),
                                details,
                            })
                        }
                        Err(e) => Err(e),
                    }?;
                }
                label.to_owned()
            };

            // Mark label on target node input
            if !label_store.contains_key(&target_id) {
                label_store.insert(target_id.clone(), NodeLabels::new());
            }
            let target_labels = label_store.get_mut(&target_id).unwrap();
            target_labels.inputs.insert(target_in.into(), label);
        }

        // Iterate over outgoing connections again, queueing
        // up outgoing nodes that do not yet have all outputs labeled.
        for edge in graph.get_connections(node, Some(PortType::Output)) {
            let (target_id, _) = edge.target();
            let target_node = graph.get_node(target_id).ok_or(NODE_NOT_FOUND)?;

            if !target_node.is_root()
                && !are_all_ports_labeled::<G, P::Label>(
                    &label_store,
                    target_node,
                    PortType::Output,
                )?
            {
                node_queue.push_back(target_node);
            }
        }
    }
    debug!("Finished processing queue.");

    // Queue empty! At this point, as only the root node may receive cyclic input,
    // the root node should have it's inputs labeled, and can compare to
    // the initial input.

    if !are_all_ports_labeled::<G, P::Label>(&label_store, root_node, PortType::Input)? {
        return Err("Root node has incomplete inputs!".into());
    }

    debug!("Finalized label cache from validation: {:#?}", label_store);

    fn are_all_ports_labeled<G: PortGraph, L: IfcLabel>(
        label_store: &LabelStore<L>,
        node: &G::Node,
        port_type: PortType,
    ) -> Result<bool> {
        let node_id = node.id();
        let Some(target_labels) = label_store.get(&node_id.to_string()) else {
            return Ok(false);
        };

        Ok(match port_type {
            PortType::Input => node
                .inputs()
                .all(|input| target_labels.inputs.contains_key(input)),
            PortType::Output => node
                .outputs()
                .all(|output| target_labels.outputs.contains_key(output)),
        })
    }
    Ok(())
}
