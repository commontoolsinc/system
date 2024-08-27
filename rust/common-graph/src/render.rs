use crate::{storage::OwnedGraphData, CommonGraphError, Graph, Node, PortType, Result};
use std::{fmt::Debug, io::Write};

/// A value `V` from a [`OwnedGraphData<V>`] that
/// can be rendered for a port.
pub trait RenderableValue {
    /// Return a string representing this value.
    fn render_value(&self) -> String;
}

impl RenderableValue for () {
    fn render_value(&self) -> String {
        String::from("()")
    }
}

/// Render options for the [`render_with_options`] method.
#[derive(Clone)]
pub struct RenderOpts<V = ()>
where
    V: RenderableValue + Clone,
{
    /// Graph name to display when rendering.
    pub graph_name: String,
    /// Style for invisible edges. By default, they're
    /// invisible, but may be customized for debugging purposes.
    pub invisible_style: String,
    /// Font string used in render output.
    pub font_name: String,
    /// Font size used in render output.
    pub font_size: u16,
    /// Background color of node objects.
    pub node_background_color: String,
    /// Background color of port objects.
    pub port_background_color: String,
    /// Background color of port groups.
    pub port_group_background_color: String,
    /// Data from a processed graph to visualize.
    pub graph_data: Option<OwnedGraphData<V>>,
}

impl<V> Default for RenderOpts<V>
where
    V: RenderableValue + Clone,
{
    fn default() -> Self {
        RenderOpts {
            graph_name: "".into(),
            invisible_style: "[style=invis]".into(),
            font_name: "Helvetica,Arial,sans-serif".into(),
            font_size: 12,
            node_background_color: "darkgrey".into(),
            port_background_color: "white".into(),
            port_group_background_color: "lightgrey".into(),
            graph_data: None,
        }
    }
}

/// Builder pattern for creating a [`RenderOpts`] via
/// [`RenderOptsBuilder::build`], or rendering directly via
/// [`RenderOptsBuilder::render`].
#[derive(Default)]
pub struct RenderOptsBuilder<V = ()>
where
    V: RenderableValue + Clone,
{
    opts: RenderOpts<V>,
}

impl<V> RenderOptsBuilder<V>
where
    V: RenderableValue + Clone,
{
    /// Set [RenderOpts]' `graph_name`.
    pub fn graph_name(mut self, graph_name: &str) -> Self {
        self.opts.graph_name = graph_name.into();
        self
    }

    /// Set [RenderOpts]' `invisible_style`.
    pub fn invisible_style(mut self, style: &str) -> Self {
        self.opts.invisible_style = style.into();
        self
    }

    /// Sets [RenderOpts]' `invisible_style` to
    /// a filled, red style for debugging.
    pub fn render_invisibles(mut self) -> Self {
        self.opts.invisible_style = "[style=filled,color=red]".into();
        self
    }

    /// Set [RenderOpts]' `font_name`.
    pub fn font_name(mut self, font_name: &str) -> Self {
        self.opts.font_name = font_name.into();
        self
    }

    /// Set [RenderOpts]' `font_size`.
    pub fn font_size(mut self, font_size: u16) -> Self {
        self.opts.font_size = font_size;
        self
    }

    /// Set [RenderOpts]' `node_background_color`.
    pub fn node_background_color(mut self, color: &str) -> Self {
        self.opts.node_background_color = color.into();
        self
    }

    /// Set [RenderOpts]' `port_background_color`.
    pub fn port_background_color(mut self, color: &str) -> Self {
        self.opts.port_background_color = color.into();
        self
    }

    /// Set [RenderOpts]' `port_group_background_color`.
    pub fn port_group_background_color(mut self, color: &str) -> Self {
        self.opts.port_group_background_color = color.into();
        self
    }

    /// Set [RenderOpts]' `graph_data`.
    pub fn graph_data(mut self, data: OwnedGraphData<V>) -> Self {
        self.opts.graph_data = Some(data);
        self
    }

    /// Return the built [RenderOpts] from this builder.
    pub fn build(self) -> RenderOpts<V> {
        self.opts
    }

    /// Call [render_with_options] with the [RenderOpts] from this builder.
    pub fn render<T: Debug, W: Write>(self, graph: &Graph<T>, w: W) -> Result<()> {
        render_with_options(graph, w, &self.opts)
    }
}

/// Render [Graph] as a DOT file to provided writer
/// with default rendering options.
/// See [render_with_options] for more.
pub fn render<T: Debug, W: Write>(graph: &Graph<T>, w: W) -> Result<()> {
    render_with_options::<_, _, ()>(graph, w, &RenderOpts::default())
}

/// Generates output in [Graphviz] [DOT] format.
///
/// Traverses the provided [Graph] and writes a DOT file into the provided
/// writer.
///
/// You can, for example, convert the dot file to an svg:
///
/// `dot -Tsvg graph.dot -o graph.svg`
///
/// [Graphviz]: https://www.graphviz.org
/// [DOT]: https://www.graphviz.org/doc/info/lang.html
pub fn render_with_options<T, W, V>(
    graph: &Graph<T>,
    mut w: W,
    options: &RenderOpts<V>,
) -> Result<()>
where
    T: Debug,
    W: Write,
    V: RenderableValue + Clone,
{
    let graph_name = &options.graph_name;
    let port_bg = &options.port_background_color;
    let font_name = &options.font_name;
    let font_size = &options.font_size;
    writeln!(
        w,
        r#"digraph {{
label = "{graph_name}";
graph [fontsize={font_size} fontname="{font_name}"];
node [style="rounded,filled" color={port_bg} shape=rectangle fontsize={font_size} fontname="{font_name}"];
edge [fontsize={font_size} fontname="{font_name}"];
rankdir = "TB";
"#
    )?;

    for (index, node) in graph.nodes().iter().enumerate() {
        render_node::<T, W, V>(&mut w, node, index, options)?;
        render_outgoing_edges(&mut w, graph, node)?;
    }

    writeln!(w, "}}")?;

    Ok(())
}

/// Render a [`Node`] and its ports as a `subgraph`.
fn render_node<T, W, V>(
    w: &mut W,
    node: &Node<T>,
    index: usize,
    options: &RenderOpts<V>,
) -> Result<()>
where
    T: Debug,
    W: Write,
    V: RenderableValue + Clone,
{
    let node_id = node.label().to_string();
    validate_slug(&node_id)?;

    writeln!(
        w,
        r#"
subgraph {node_id} {{
  label = "{node_id}";
  style = "rounded,filled";
  color = {};
  cluster = true;
"#,
        options.node_background_color
    )?;

    let input_slugs = render_ports::<T, W, V>(w, node, index, options, PortType::Input)?;
    let output_slugs = render_ports::<T, W, V>(w, node, index, options, PortType::Output)?;

    // Render invisible connections between an input
    // and an output port to create a hierarchy.
    if let (Some(input_slug), Some(output_slug)) = (input_slugs.first(), output_slugs.first()) {
        writeln!(
            w,
            r#"  {} -> {} {};"#,
            input_slug, output_slug, options.invisible_style
        )?;
    }

    writeln!(w, "}}")?;

    Ok(())
}

/// Renders [`Graph::Edge`] where the edge source
/// is an output port from `node`.
fn render_outgoing_edges<T: Debug, W: Write>(
    w: &mut W,
    graph: &Graph<T>,
    node: &Node<T>,
) -> Result<()> {
    let node_id = node.label();
    for (port, outgoing_ports) in node.outputs() {
        let Some(outgoing_ports) = outgoing_ports else {
            continue;
        };
        for (out_index, out_port) in outgoing_ports {
            let out_node = graph.get_node(*out_index)?;
            writeln!(
                w,
                r#""{}" -> "{}";"#,
                slugify_port(node_id, port, &PortType::Output)?,
                slugify_port(out_node.label(), out_port, &PortType::Input)?
            )?;
        }
    }
    Ok(())
}

/// Renders a group of `node`'s ports as a `subgraph`.
fn render_ports<T, W, V>(
    w: &mut W,
    node: &Node<T>,
    index: usize,
    options: &RenderOpts<V>,
    port_type: PortType,
) -> Result<Vec<String>>
where
    T: Debug,
    W: Write,
    V: RenderableValue + Clone,
{
    let node_id = node.label().to_string();
    let (ports, rank) = match port_type {
        PortType::Input => (
            node.inputs()
                .iter()
                .map(|(name, _)| name.to_string())
                .collect::<Vec<_>>(),
            "source",
        ),
        PortType::Output => (
            node.outputs()
                .iter()
                .map(|(name, _)| name.to_string())
                .collect::<Vec<_>>(),
            "same",
        ),
    };

    // Open anonymous subgraph,
    // set rank for inputs to order above outputs,
    // and override `label`, otherwise repeats the Node label.
    writeln!(
        w,
        r#"
  subgraph {{
    rank = {};
    label = "{}";
    color = "{}";
"#,
        rank, port_type, options.port_group_background_color
    )?;

    let mut port_slugs = vec![];
    for port in ports {
        let port_slug = slugify_port(&node_id, &port, &port_type)?;

        let label = if let Some(render_data) = get_render_data(index, &port, options, &port_type) {
            format!("{} ({})", port, render_data)
        } else {
            port
        };

        writeln!(w, r#"    "{}" [label="{}"];"#, port_slug, label)?;
        port_slugs.push(format!(r#""{}""#, port_slug));
    }
    // Close anonymous subgraph
    writeln!(w, "  }}")?;

    Ok(port_slugs)
}

/// Renders a group of `node`'s ports as a `subgraph`.
fn get_render_data<V>(
    index: usize,
    port_name: &str,
    options: &RenderOpts<V>,
    port_type: &PortType,
) -> Option<String>
where
    V: RenderableValue + Clone,
{
    let Some(options_data) = options.graph_data.as_ref() else {
        return None;
    };
    let graph_data = options_data.inner();
    let Some(data) = graph_data.get(index) else {
        return None;
    };
    let port_data = match port_type {
        PortType::Input => &data.0,
        PortType::Output => &data.1,
    };
    let Some((_, value)) = port_data.iter().find(|(k, _)| *k == port_name) else {
        return None;
    };
    Some(match value {
        Some(v) => v.render_value(),
        None => String::from("None"),
    })
}
fn slugify_port(node_id: &str, port: &str, port_type: &PortType) -> Result<String> {
    let slug = format!("{}__{}__{}", node_id, port_type, port);
    validate_slug(&slug)?;
    Ok(slug)
}

/// Validates a generated slug for compatibility with dot.
/// The first character must be an alphanumeric or '_',
/// and all remaining characters must be alphanumeric or '_'.
fn validate_slug(slug: &str) -> Result<()> {
    match slug.chars().next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => {
            return Err(CommonGraphError::InternalError(format!(
                "Invalid slug: {}",
                slug
            )))
        }
    }
    if !slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(CommonGraphError::InternalError(format!(
            "Invalid slug: {}",
            slug
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::GraphBuilder;
    use super::*;

    #[test]
    fn it_renders_to_dot_format() -> Result<()> {
        let builder = GraphBuilder::default();
        let graph = builder
            .set_label("Storage")
            .set_graph_input(vec![
                "event_reset",
                "event_confirm",
                "todos",
                "api_key",
                "journal",
            ])
            .set_graph_output(vec![
                "event_reset",
                "event_confirm",
                "todos",
                "api_key",
                "journal",
            ])
            .node(
                "PromptUI",
                (),
                vec!["todos", "journal", "reset_trigger", "confirm_trigger"],
                vec!["prompt", "rendertree"],
            )
            .node("LLM", (), vec!["prompt", "api_key"], vec!["dream"])
            .node("ConfirmUI", (), vec!["dream"], vec!["rendertree"])
            .node(
                "RenderSink",
                (),
                vec!["rendertree1", "rendertree2"],
                vec!["event_reset", "event_confirm"],
            )
            .connect_input("event_reset", ("PromptUI", "reset_trigger"))?
            .connect_input("event_confirm", ("PromptUI", "confirm_trigger"))?
            .connect_input("todos", ("PromptUI", "todos"))?
            .connect_input("journal", ("PromptUI", "journal"))?
            .connect_input("api_key", ("LLM", "api_key"))?
            .connect(("PromptUI", "prompt"), ("LLM", "prompt"))?
            .connect(("LLM", "dream"), ("ConfirmUI", "dream"))?
            .connect(("PromptUI", "rendertree"), ("RenderSink", "rendertree1"))?
            .connect(("ConfirmUI", "rendertree"), ("RenderSink", "rendertree2"))?
            .connect_output(("RenderSink", "event_reset"), "event_reset")?
            .connect_output(("RenderSink", "event_confirm"), "event_confirm")?
            .build()?;

        let mut out = std::io::Cursor::new(vec![]);
        RenderOptsBuilder::<()>::default()
            .graph_name("IFC Example")
            .render(&graph, &mut out)?;
        let dot = String::from_utf8(out.into_inner())?;
        assert!(dot.contains("subgraph Storage"));
        Ok(())
    }
}
