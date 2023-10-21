use crate::{prelude::*, support::fields_eq};
use truc::record::type_resolver::TypeResolver;

#[derive(Getters)]
pub struct Dedup {
    name: FullyQualifiedName,
    #[getset(get = "pub")]
    inputs: [NodeStream; 1],
    #[getset(get = "pub")]
    outputs: [NodeStream; 1],
}

impl Dedup {
    fn new<R: TypeResolver + Copy>(
        graph: &mut GraphBuilder<R>,
        name: FullyQualifiedName,
        inputs: [NodeStream; 1],
    ) -> Self {
        let mut streams = StreamsBuilder::new(&name, &inputs);
        streams
            .output_from_input(0, true, graph)
            .pass_through(|_| {});
        let outputs = streams.build();
        Self {
            name,
            inputs,
            outputs,
        }
    }
}

impl DynNode for Dedup {
    fn name(&self) -> &FullyQualifiedName {
        &self.name
    }

    fn inputs(&self) -> &[NodeStream] {
        &self.inputs
    }

    fn outputs(&self) -> &[NodeStream] {
        &self.outputs
    }

    fn gen_chain(&self, graph: &Graph, chain: &mut Chain) {
        let record = chain
            .stream_definition_fragments(self.inputs.single())
            .record();
        let record_definition = &graph.record_definitions()[self.inputs.single().record_type()];
        let variant = &record_definition[self.inputs.single().variant_id()];

        let eq = fields_eq(&record, variant.data().map(|d| record_definition[d].name()));

        let inline_body = quote! {
            datapet_support::iterator::dedup::Dedup::new(input, #eq)
        };

        chain.implement_inline_node(
            self,
            self.inputs.single(),
            self.outputs.single(),
            &inline_body,
        );
    }

    fn all_nodes(&self) -> Box<dyn Iterator<Item = &dyn DynNode> + '_> {
        Box::new(Some(self as &dyn DynNode).into_iter())
    }
}

pub fn dedup<R: TypeResolver + Copy>(
    graph: &mut GraphBuilder<R>,
    name: FullyQualifiedName,
    inputs: [NodeStream; 1],
) -> Dedup {
    Dedup::new(graph, name, inputs)
}
