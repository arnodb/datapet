use crate::{prelude::*, stream::UniqueNodeStream};
use proc_macro2::TokenStream;
use truc::record::{
    definition::{RecordDefinition, RecordVariant},
    type_resolver::TypeResolver,
};

#[derive(Getters)]
struct InPlaceFilter {
    name: FullyQualifiedName,
    #[getset(get = "pub")]
    inputs: [NodeStream; 1],
    #[getset(get = "pub")]
    outputs: [NodeStream; 1],
}

impl InPlaceFilter {
    fn new<R: TypeResolver + Copy>(
        graph: &mut GraphBuilder<R>,
        name: FullyQualifiedName,
        inputs: [NodeStream; 1],
    ) -> Self {
        let mut streams = StreamsBuilder::new(&name, &inputs);
        streams.output_from_input(0, graph).pass_through();
        let outputs = streams.build();
        Self {
            name,
            inputs,
            outputs,
        }
    }

    fn gen_chain<B>(&self, graph: &Graph, chain: &mut Chain, body: B)
    where
        B: FnOnce(&RecordDefinition, &RecordVariant) -> TokenStream,
    {
        let thread = chain.get_thread_id_and_module_by_source(
            self.inputs.unique(),
            &self.name,
            self.outputs.some_unique(),
        );

        let def = chain.stream_definition_fragments(self.outputs.unique());

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

            let record = def.record();

            let input = thread.format_input(
                self.inputs.unique().source(),
                graph.chain_customizer(),
                &mut import_scope,
            );

            let record_definition = &graph.record_definitions()[self.inputs.unique().record_type()];
            let record_variant = &record_definition[self.inputs.unique().variant_id()];
            let body = body(record_definition, record_variant);

            let fn_def = quote! {
                pub fn #fn_name(#[allow(unused_mut)] mut thread_control: #thread_module::ThreadControl) -> impl FallibleIterator<Item = #record, Error = #error_type> {
                    #input
                    input.map(|mut record| {
                        #body
                        Ok(record)
                    })
                }
            };
            scope.raw(&fn_def.to_string());
        }

        import_scope.import(scope, graph.chain_customizer());
    }

    fn gen_chain_simple<'f, F>(
        &self,
        graph: &Graph,
        chain: &mut Chain,
        fields: F,
        transform: TokenStream,
    ) where
        F: IntoIterator<Item = &'f str> + Clone,
    {
        self.gen_chain(graph, chain, |record_definition, variant| {
            let data = variant
                .data()
                .filter_map(|d| {
                    let datum = &record_definition[d];
                    if fields
                        .clone()
                        .into_iter()
                        .any(|field| field == datum.name())
                    {
                        Some(datum)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let mut_fields = data
                .iter()
                .map(|datum| format_ident!("{}_mut", datum.name()));
            let fields = data.iter().map(|datum| format_ident!("{}", datum.name()));
            quote! {
                #(*record.#mut_fields() = record.#fields()#transform.into();)*
            }
        });
    }
}

pub mod string {
    use super::InPlaceFilter;
    use crate::graph::{DynNode, GraphBuilder};
    use crate::support::FullyQualifiedName;
    use crate::{chain::Chain, graph::Graph, stream::NodeStream};
    use std::ops::Deref;
    use truc::record::type_resolver::TypeResolver;

    pub struct ToLowercase {
        in_place: InPlaceFilter,
        fields: Box<[Box<str>]>,
    }

    impl ToLowercase {
        pub fn inputs(&self) -> &[NodeStream; 1] {
            self.in_place.inputs()
        }

        pub fn outputs(&self) -> &[NodeStream; 1] {
            self.in_place.outputs()
        }
    }

    impl DynNode for ToLowercase {
        fn gen_chain(&self, graph: &Graph, chain: &mut Chain) {
            self.in_place.gen_chain_simple(
                graph,
                chain,
                self.fields.iter().map(Box::as_ref),
                quote! { .to_lowercase() },
            );
        }
    }

    pub fn to_lowercase<R: TypeResolver + Copy>(
        graph: &mut GraphBuilder<R>,
        name: FullyQualifiedName,
        inputs: [NodeStream; 1],
        fields: &[&str],
    ) -> ToLowercase {
        ToLowercase {
            in_place: InPlaceFilter::new(graph, name, inputs),
            fields: fields
                .iter()
                .map(Deref::deref)
                .map(Into::into)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    pub struct ReverseChars {
        in_place: InPlaceFilter,
        fields: Box<[Box<str>]>,
    }

    impl ReverseChars {
        pub fn inputs(&self) -> &[NodeStream; 1] {
            self.in_place.inputs()
        }

        pub fn outputs(&self) -> &[NodeStream; 1] {
            self.in_place.outputs()
        }
    }

    impl DynNode for ReverseChars {
        fn gen_chain(&self, graph: &Graph, chain: &mut Chain) {
            self.in_place.gen_chain_simple(
                graph,
                chain,
                self.fields.iter().map(Box::as_ref),
                quote! { .chars().rev().collect::<String>() },
            );
        }
    }

    pub fn reverse_chars<R: TypeResolver + Copy>(
        graph: &mut GraphBuilder<R>,
        name: FullyQualifiedName,
        inputs: [NodeStream; 1],
        fields: &[&str],
    ) -> ReverseChars {
        ReverseChars {
            in_place: InPlaceFilter::new(graph, name, inputs),
            fields: fields
                .iter()
                .map(Deref::deref)
                .map(Into::into)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }
}
