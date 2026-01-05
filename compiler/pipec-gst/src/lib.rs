use pipec_arena::AStr;
use pipec_arena::{ASlice, Arena};
use pipec_arena_structures::ListNode;
use pipec_ast::ast::ASTNode;
use pipec_ast::ast::FunctionDeclarationParameters;
use pipec_ast::ast::Path;
use pipec_ast::ast::PathNode;
use pipec_ast::ast::asttree::ASTTree;
use pipec_file_loader::FileLoader;
use pipec_span::Span;
use std::collections::HashMap;

pub struct GlobalSymbolTree<'this> {
    ast: ASTTree,
    loader: &'this mut FileLoader,
    arena: &'this mut Arena,
    pub map: HashMap<SymbolName, Symbol>,
    context: SymbolName,
    src: ASlice<AStr>,
}

impl<'this> GlobalSymbolTree<'this> {
    pub fn new(arena: &'this mut Arena, loader: &'this mut FileLoader, ast: ASTTree) -> Self {
        let map = HashMap::new();
        let src = loader.load(ast.id);
        let context = SymbolName::new();
        Self {
            ast,
            arena,
            loader,
            map,
            src,
            context,
        }
    }

    pub fn generate(&mut self) {
        loop {
            let next = self.ast.next_node(self.arena);
            match next {
                Some(v) => match v {
                    ASTNode::EOF => {
                        break;
                    }
                    _ => self.check_node(v.clone()),
                },
                None => break,
            }
        }
    }

    pub(crate) fn check_node(&mut self, input: ASTNode) {
        match input {
            ASTNode::FunctionDeclaration {
                name,
                params,
                block: _,
                out_type,
            } => self.parse_function_declaration(name, params, out_type),
            ASTNode::ViewportDeclaration {
                name,
                params,
                block: _,
            } => self.parse_viewport_declaration(name, params),
            ASTNode::ModStatement { name, tree } => {
                self.parse_mod_statement(name, tree);
            }
            _ => {}
        }
    }

    pub(crate) fn parse_function_declaration(
        &mut self,
        name: Span,
        params: FunctionDeclarationParameters,
        out_type: Option<Path>,
    ) {
        let return_type = match out_type {
            None => Type::Nothing,
            Some(v) => self.type_from_path(&v),
        };
        let params = self.ast_to_gst_params(params);
        let symbol = Symbol::Function {
            params,
            return_type,
        };
        let mut cloned = self.context.clone();
        let name = name.parse_arena(self.src, self.arena).to_string();
        cloned.path.push(name);

        self.map.insert(cloned, symbol);
    }

    pub(crate) fn parse_viewport_declaration(
        &mut self,
        name: Span,
        params: FunctionDeclarationParameters,
    ) {
        let params = self.ast_to_gst_params(params);
        let symbol = Symbol::Viewport { params };
        let mut cloned = self.context.clone();
        let name = name.parse_arena(self.src, self.arena).to_string();
        cloned.path.push(name);

        self.map.insert(cloned, symbol);
    }

    pub(crate) fn parse_mod_statement(&mut self, name: Span, mut tree: ASTTree) {
        let name = name.parse_arena(self.src, self.arena).to_string();
        let old_src = self.src;
        let old_context = self.context.clone();
        let mut mod_context = self.context.clone();
        mod_context.path.push(name);
        self.context = mod_context;
        self.src = self.loader.load(tree.id);
        loop {
            let next = tree.next_node(self.arena);
            match next {
                Some(v) => self.check_node(v.clone()),
                _ => break,
            }
        }
        self.context = old_context;
        self.src = old_src;
    }
    pub(crate) fn type_from_path(&mut self, input: &Path) -> Type {
        let vec = input.0.clone();
        use Type::*;

        if vec.len_eq(1, self.arena) {
            let first = vec.first(self.arena);
            match first {
                ListNode::Empty => {}
                ListNode::Node(PathNode::Named { name, param: _ }, _) => {
                    let name = name.parse_arena(self.src, self.arena);
                    match name {
                        "integer8" => return Integer8,
                        "unsigned8" => return Unsigned8,
                        "float8" => return Float8,
                        "integer16" => return Integer16,
                        "unsigned16" => return Unsigned16,
                        "float16" => return Float16,
                        "integer32" => return Integer32,
                        "unsigned32" => return Unsigned32,
                        "float32" => return Float32,
                        "integer64" => return Integer64,
                        "unsigned64" => return Unsigned64,
                        "float64" => return Float64,
                        "floatport" => return FloatPort,
                        "nothing" => return Nothing,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Link(self.path_to_symbol_name(input))
    }

    pub(crate) fn ast_to_gst_params(
        &mut self,
        input: FunctionDeclarationParameters,
    ) -> FunctionParameters {
        let avec = self.arena.take(input.0);
        let mut out = FunctionParameters(Vec::new());
        for i in avec.iter() {
            let name = i.name.parse_arena(self.src, self.arena).to_string();
            let p_type = self.type_from_path(&i.arg_type);
            out.0.push((name, p_type));
        }
        out
    }

    pub(crate) fn path_to_symbol_name(&mut self, input: &Path) -> SymbolName {
        let vec = input.0.clone();
        let mut out = SymbolName::new();
        let mut iter = vec.iter(self.arena);
        loop {
            let next = iter.next();
            if next.is_none() {
                break;
            }
            if let Some(PathNode::Named { name, param: _ }) = next {
                let parsed = name.parse_arena(self.src, self.arena).to_string();
                out.path.push(parsed);
            }
        }
        out
    }
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct SymbolName {
    pub(crate) path: Vec<String>,
}

impl SymbolName {
    fn new() -> Self {
        Self { path: vec![] }
    }
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub enum Symbol {
    Function {
        return_type: Type,
        params: FunctionParameters,
    },
    Viewport {
        params: FunctionParameters,
    },
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub enum Type {
    Integer8,
    Unsigned8,
    Float8,
    Integer16,
    Unsigned16,
    Float16,
    Integer32,
    Unsigned32,
    Float32,
    Integer64,
    Unsigned64,
    Float64,
    FloatPort,
    Nothing,
    Link(SymbolName),
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct FunctionParameters(Vec<(String, Type)>);
