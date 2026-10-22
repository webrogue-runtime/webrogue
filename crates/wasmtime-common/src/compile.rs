use anyhow::Context as _;
use wac_graph::{CompositionGraph, EncodeOptions, PackageId, plug, types::Package};

pub fn process_component(component_bytes: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let mut graph = CompositionGraph::new();

    let virt_pkg = Package::from_bytes(
        "webrogue-virt",
        None,
        include_bytes!("../../../vfs-adapter/webrogue_vfs_adapter.wasm"),
        graph.types_mut(),
    )
    .context("failed to decode virtualized adapter package")?;

    let virt = graph.register_package(virt_pkg)?;

    let component_pkg = Package::from_bytes(
        "component",
        None,
        component_bytes.clone(),
        graph.types_mut(),
    )
    .context("failed to decode component package")?;
    let component = graph.register_package(component_pkg)?;

    // The adapter is only plugged into components that use the filesystem,
    // there is nothing to virtualize otherwise.
    if !uses_filesystem(&graph, component) {
        return Ok(component_bytes);
    }

    plug(&mut graph, vec![virt], component)
                .with_context(|| "Unable to plug virtualized adapter into component.\nMake sure virtualizations are enabled and being used.")?;

    Ok(graph
        .encode(EncodeOptions::default())
        .context("failed to encode composed component")?)
}

/// Whether the component uses the filesystem.
fn uses_filesystem(graph: &CompositionGraph, component: PackageId) -> bool {
    graph.types()[graph[component].ty()]
        .imports
        .keys()
        .any(|name| name.starts_with("wasi:filesystem/"))
}
