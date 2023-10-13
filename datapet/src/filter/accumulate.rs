use crate::{prelude::*, stream::UniqueNodeStream};
use truc::record::type_resolver::TypeResolver;

#[derive(Getters)]
pub struct Accumulate {
    name: FullyQualifiedName,
    #[getset(get = "pub")]
    inputs: [NodeStream; 1],
    #[getset(get = "pub")]
    outputs: [NodeStream; 1],
}

impl Accumulate {
    fn new<R: TypeResolver + Copy>(
        graph: &mut GraphBuilder<R>,
        name: FullyQualifiedName,
        inputs: [NodeStream; 1],
    ) -> Self {
        let mut streams = StreamsBuilder::new(&name, &inputs);
        streams.output_from_input(0, true, graph).pass_through();
        let outputs = streams.build(graph);

        Self {
            name,
            inputs,
            outputs,
        }
    }
}

impl DynNode for Accumulate {
    fn name(&self) -> &FullyQualifiedName {
        &self.name
    }

    fn inputs(&self) -> &[NodeStream] {
        &self.inputs
    }

    fn outputs(&self) -> &[NodeStream] {
        &self.outputs
    }

    fn gen_chain(&self, _graph: &Graph, chain: &mut Chain) {
        let inline_body = quote! {
            datapet_support::iterator::accumulate::Accumulate::new(input)
        };

        chain.implement_inline_node(
            self,
            self.inputs.unique(),
            self.outputs.unique(),
            &inline_body,
        );
    }

    fn all_nodes(&self) -> Box<dyn Iterator<Item = &dyn DynNode> + '_> {
        Box::new(Some(self as &dyn DynNode).into_iter())
    }
}

pub fn accumulate<R: TypeResolver + Copy>(
    graph: &mut GraphBuilder<R>,
    name: FullyQualifiedName,
    inputs: [NodeStream; 1],
) -> Accumulate {
    Accumulate::new(graph, name, inputs)
}
