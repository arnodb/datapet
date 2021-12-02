use crate::{
    chain::{Chain, ImportScope},
    dyn_node,
    graph::{DynNode, Graph, GraphBuilder, Node},
    stream::NodeStream,
    support::FullyQualifiedName,
};

pub struct Sink {
    name: FullyQualifiedName,
    inputs: [NodeStream; 1],
    debug: Option<String>,
}

impl Node<1, 0> for Sink {
    fn inputs(&self) -> &[NodeStream; 1] {
        &self.inputs
    }

    fn outputs(&self) -> &[NodeStream; 0] {
        &[]
    }

    fn gen_chain(&self, graph: &Graph, chain: &mut Chain) {
        let thread = chain.get_thread_id_and_module_by_source(self.inputs[0].source(), &self.name);

        let local_name = self.name.last().expect("local name");
        let scope = chain.get_or_new_module_scope(
            self.name.iter().take(self.name.len() - 1),
            graph.chain_customizer(),
            thread.thread_id,
        );
        let mut import_scope = ImportScope::default();
        let node_fn = scope
            .new_fn(local_name)
            .vis("pub")
            .arg(
                "#[allow(unused_mut)] mut thread_control",
                format!("thread_{}::ThreadControl", thread.thread_id),
            )
            .ret("impl FnOnce() -> Result<(), DatapetError>");
        let input = thread.format_input(
            self.inputs[0].source(),
            graph.chain_customizer(),
            &mut import_scope,
        );
        crate::chain::fn_body(
            format!(
                r#"move || {{
    {input}
    let mut input = input;
    let mut read = 0;
    while let Some(record) = input.next()? {{"#,
                input = input,
            ),
            node_fn,
        );
        if let Some(debug) = &self.debug {
            crate::chain::fn_body(debug, node_fn);
        }
        crate::chain::fn_body(
            format!(
                r#"
        read += 1;
    }}
    println!("read {name} {{}}", read);
    Ok(())
}}"#,
                name = self.name,
            ),
            node_fn,
        );
        import_scope.import(scope, graph.chain_customizer());

        chain.set_thread_main(thread.thread_id, self.name.clone());
    }
}

dyn_node!(Sink);

pub fn sink(
    _graph: &mut GraphBuilder,
    name: FullyQualifiedName,
    input: NodeStream,
    debug: Option<String>,
) -> Sink {
    Sink {
        name,
        inputs: [input],
        debug,
    }
}
