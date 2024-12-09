use crate::{Entry, Error, HashDisplay, Node, NodeExt, Result, Storage, Tree};
use std::io::Write;

trait Renderable<const P: u8> {
    fn get_id_and_label(&self) -> Result<(String, String)>;
}

impl<const P: u8> Renderable<P> for Node<P> {
    fn get_id_and_label(&self) -> Result<(String, String)> {
        let id = format!(
            "node_{}",
            HashDisplay::from(self.hash().to_owned()).to_string()
        );
        let label = format!(
            "{:.10} R={}",
            //HashDisplay::from(self.boundary().to_owned()).to_string(),
            HashDisplay::from(self.hash().to_owned()).to_string(),
            self.rank()
        );
        Ok((id, label))
    }
}

impl<const P: u8> Renderable<P> for Entry {
    fn get_id_and_label(&self) -> Result<(String, String)> {
        let key = HashDisplay::from(self.key.to_owned()).to_string();
        let id = format!("node_{}", &key);
        let label = format!("{:.10} R={}", &key, self.rank(P as u32));
        Ok((id, label))
    }
}

/// Generates output in [Graphviz] [DOT] format.
///
/// Traverses the provided [Tree] and writes a DOT file into the provided
/// writer.
///
/// You can, for example, convert the dot file to an svg:
///
/// `dot -Tsvg tree.dot -o tree.svg`
///
/// [Graphviz]: https://www.graphviz.org
/// [DOT]: https://www.graphviz.org/doc/info/lang.html
pub async fn render<const P: u8, S, W>(tree: &Tree<P, S>, w: W) -> Result<()>
where
    S: Storage,
    W: Write,
{
    let Some(root) = tree.root() else {
        return Err(Error::Internal("Empty tree.".into()));
    };

    render_node::<P, S, W>(root, tree.storage(), w).await
}

/// Renders a graph where `node` is the root node.
///
/// See [`render`].
pub async fn render_node<const P: u8, S, W>(node: &Node<P>, storage: &S, mut w: W) -> Result<()>
where
    S: Storage,
    W: Write,
{
    let graph_name = "Ranked Prolly Tree";
    let port_bg = "lightgrey";
    let font_name = "Helvetica,Arial,sans-serif";
    let font_size = 12;
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

    let mut nodes = vec![node.to_owned()];
    loop {
        nodes = render_nodes(nodes, storage, &mut w).await?;
        if nodes.is_empty() {
            break;
        }
    }

    writeln!(w, "}}")?;

    Ok(())
}

/// Render a [`Node`] and its ports as a `subgraph`.
async fn render_nodes<const P: u8, S, W>(
    nodes: Vec<Node<P>>,
    storage: &S,
    w: &mut W,
) -> Result<Vec<Node<P>>>
where
    S: Storage,
    W: Write,
{
    let mut out_nodes = vec![];
    for node in nodes {
        let (node_id, node_label) = node.get_id_and_label()?;
        writeln!(w, "{} [label = \"{}\"];", &node_id, &node_label)?;

        match node.is_branch() {
            true => {
                let children = node.into_children(storage).await?;
                for child in children {
                    let (child_id, _) = child.get_id_and_label()?;
                    writeln!(w, "{} -> {};", &node_id, child_id)?;
                    out_nodes.push(child);
                }
            }
            false => {
                for entry in node.into_entries()? {
                    let (entry_id, entry_label) =
                        <Entry as Renderable<P>>::get_id_and_label(&entry)?;
                    writeln!(w, "{} [label = \"{}\"];", &entry_id, &entry_label)?;
                    writeln!(w, "{} -> {};", &node_id, &entry_id)?;
                }
            }
        }
    }
    Ok(out_nodes)
}

#[cfg(all(not(target_arch = "wasm32"), test))]
mod tests {
    use super::*;
    use crate::{BincodeEncoder, NodeStorage, SyncMemoryStore, Tree};
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn it_renders_to_dot_format() -> Result<()> {
        let mut set = BTreeMap::default();
        for i in 0..1024u32 {
            let key = i.to_be_bytes().to_vec();
            let value = <[u8; 32] as From<blake3::Hash>>::from(blake3::hash(&key)).to_vec();
            set.insert(key, value);
        }
        let storage = NodeStorage::new(BincodeEncoder::default(), SyncMemoryStore::default());
        let tree = Tree::<32, _>::from_set(set, storage.clone()).await?;

        let mut out = std::io::Cursor::new(vec![]);
        render(&tree, &mut out).await?;
        let _dot = String::from_utf8(out.into_inner())?;
        //std::fs::write("tree.dot", &dot)?;
        Ok(())
    }
}
