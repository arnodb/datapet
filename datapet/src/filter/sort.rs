use crate::{prelude::*, support::fields_cmp};

#[derive(Getters)]
pub struct Sort {
    name: FullyQualifiedName,
    #[getset(get = "pub")]
    inputs: [NodeStream; 1],
    #[getset(get = "pub")]
    outputs: [NodeStream; 1],
    fields: Vec<String>,
}

impl Sort {
    fn new(
        graph: &mut GraphBuilder,
        name: FullyQualifiedName,
        inputs: [NodeStream; 1],
        fields: &[&str],
    ) -> Self {
        let mut streams = StreamsBuilder::new(&name, &inputs);
        streams.output_from_input(0, graph).pass_through();
        let outputs = streams.build();
        Self {
            name,
            inputs,
            outputs,
            fields: fields.iter().map(ToString::to_string).collect::<Vec<_>>(),
        }
    }
}

impl DynNode for Sort {
    fn gen_chain(&self, graph: &Graph, chain: &mut Chain) {
        let thread = chain.get_thread_id_and_module_by_source(self.inputs[0].source(), &self.name);

        let scope = chain.get_or_new_module_scope(
            self.name.iter().take(self.name.len() - 1),
            graph.chain_customizer(),
            thread.thread_id,
        );
        let mut import_scope = ImportScope::default();
        import_scope.add_import_with_error_type("fallible_iterator", "FallibleIterator");

        {
            let fn_name = format_ident!("{}", **self.name.last().expect("local name"));
            let thread_module = format_ident!("thread_{}", thread.thread_id);
            let error_type = graph.chain_customizer().error_type.to_name();

            let def =
                self.outputs[0].definition_fragments(&graph.chain_customizer().streams_module_name);
            let record = def.record();

            let input = thread.format_input(
                self.inputs[0].source(),
                graph.chain_customizer(),
                &mut import_scope,
            );

            let cmp = fields_cmp(self.fields.iter().map(String::as_str));

            let fn_def = quote! {
                  pub fn #fn_name(#[allow(unused_mut)] mut thread_control: #thread_module::ThreadControl) -> impl FallibleIterator<Item = #record, Error = #error_type> {
                      #input
                      datapet_support::iterator::sort::Sort::new(
                          input,
                          #cmp,
                      )
                  }
            };
            scope.raw(&fn_def.to_string());
        }

        import_scope.import(scope, graph.chain_customizer());

        chain.update_thread_single_stream(thread.thread_id, &self.outputs[0]);
    }
}

pub fn sort(
    graph: &mut GraphBuilder,
    name: FullyQualifiedName,
    inputs: [NodeStream; 1],
    fields: &[&str],
) -> Sort {
    Sort::new(graph, name, inputs, fields)
}
