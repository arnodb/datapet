use crate::prelude::*;
use itertools::Itertools;
use proc_macro2::TokenStream;
use truc::record::type_resolver::TypeResolver;

#[derive(Getters)]
pub struct FunctionSource {
    name: FullyQualifiedName,
    #[getset(get = "pub")]
    inputs: [NodeStream; 0],
    #[getset(get = "pub")]
    outputs: [NodeStream; 1],
    fields: Vec<(String, String)>,
    func: String,
}

impl FunctionSource {
    fn new<R: TypeResolver + Copy>(
        graph: &mut GraphBuilder<R>,
        name: FullyQualifiedName,
        inputs: [NodeStream; 0],
        fields: &[(&str, &str)],
        func: &str,
    ) -> Self {
        let mut streams = StreamsBuilder::new(&name, &inputs);
        streams.new_main_stream(graph);

        {
            let output_stream = streams.new_main_output(graph).for_update();

            let mut output_stream_def = output_stream.borrow_mut();
            for (name, r#type) in fields.iter() {
                output_stream_def.add_dynamic_datum(*name, r#type);
            }
        }

        let outputs = streams.build();

        Self {
            name,
            inputs,
            outputs,
            fields: fields
                .iter()
                .map(|(name, r#type)| ((*name).to_owned(), (*r#type).to_owned()))
                .collect(),
            func: func.to_owned(),
        }
    }
}

impl DynNode for FunctionSource {
    fn gen_chain(&self, graph: &Graph, chain: &mut Chain) {
        let thread_id = chain.new_thread(
            self.name.clone(),
            self.inputs.to_vec().into_boxed_slice(),
            self.outputs.to_vec().into_boxed_slice(),
            None,
            false,
            Some(self.name.clone()),
        );

        let scope = chain.get_or_new_module_scope(
            self.name.iter().take(self.name.len() - 1),
            graph.chain_customizer(),
            thread_id,
        );

        {
            let fn_name = format_ident!("{}", **self.name.last().expect("local name"));
            let thread_module = format_ident!("thread_{}", thread_id);
            let error_type = graph.chain_customizer().error_type.to_name();

            let def =
                self.outputs[0].definition_fragments(&graph.chain_customizer().streams_module_name);
            let record = def.record();
            let unpacked_record = def.unpacked_record();

            let new_record_args = syn::parse_str::<syn::Expr>(
                &self
                    .fields
                    .iter()
                    .map(|(name, r#type)| format!("{}: {}", name, r#type))
                    .join(", "),
            )
            .expect("new_record_args");
            let new_record_fields = syn::parse_str::<syn::Expr>(
                &self.fields.iter().map(|(name, _type)| name).join(", "),
            )
            .expect("new_record_fields");

            let func_body: TokenStream = self.func.parse().expect("function body");

            let fn_def = quote! {
                pub fn #fn_name(mut thread_control: #thread_module::ThreadControl) -> impl FnOnce() -> Result<(), #error_type> {
                    move || {
                        let out = thread_control.output_0.take().expect("output 0");
                        let new_record = |#new_record_args| {
                            #record::new(#unpacked_record { #new_record_fields })
                        };
                        #func_body
                    }
                }
            };
            scope.raw(&fn_def.to_string());
        }
    }
}

pub fn function_source<R: TypeResolver + Copy>(
    graph: &mut GraphBuilder<R>,
    name: FullyQualifiedName,
    inputs: [NodeStream; 0],
    fields: &[(&str, &str)],
    func: &str,
) -> FunctionSource {
    FunctionSource::new(graph, name, inputs, fields, func)
}
