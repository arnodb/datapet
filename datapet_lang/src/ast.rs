#[derive(Debug)]
pub struct Module {
    pub items: Vec<ModuleItem>,
}

#[derive(Debug)]
pub enum ModuleItem {
    UseDeclaration(UseDeclaration),
    GraphDefinition(GraphDefinition),
}

impl From<UseDeclaration> for ModuleItem {
    fn from(value: UseDeclaration) -> Self {
        ModuleItem::UseDeclaration(value)
    }
}

impl From<GraphDefinition> for ModuleItem {
    fn from(value: GraphDefinition) -> Self {
        ModuleItem::GraphDefinition(value)
    }
}

#[derive(Debug)]
pub struct UseDeclaration {
    pub use_tree: UseTree,
}

#[derive(Debug)]
pub enum UseTree {
    Glob(String),
    Group(String, Vec<UseTree>),
    Path(String),
}

#[derive(Debug)]
pub struct GraphDefinition {
    pub signature: GraphDefinitionSignature,
    pub stream_lines: Vec<StreamLine>,
    pub visible: bool,
}

#[derive(Debug)]
pub struct GraphDefinitionSignature {
    pub inputs: Option<Vec<String>>,
    pub name: String,
    pub params: Vec<String>,
    pub outputs: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct StreamLine {
    pub filters: Vec<ConnectedFilter>,
    pub output: Option<StreamLineOutput>,
}

#[derive(Debug)]
pub struct ConnectedFilter {
    pub inputs: Vec<StreamLineInput>,
    pub filter: Filter,
}

#[derive(Debug)]
pub struct Filter {
    pub name: String,
    pub alias: Option<String>,
    pub params: Vec<FilterParam>,
    pub extra_outputs: Vec<String>,
}

#[derive(Debug)]
pub enum FilterParam {
    Single(String),
    Array(Vec<String>),
}

#[derive(Debug)]
pub enum StreamLineInput {
    Main,
    Named(String),
}

#[derive(Debug)]
pub enum StreamLineOutput {
    Main,
    Named(String),
}
