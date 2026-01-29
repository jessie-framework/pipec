use pipec_arena::AStr;
use pipec_arena::{ASlice, Arena};
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
    src: ASlice<AStr>,
}

#[derive(Default, Debug)]
pub struct ModuleScope<'a> {
    symbols: HashMap<&'a str, Symbol>,
    submodules: HashMap<&'a str, Self>,
}

impl<'this> GlobalSymbolTree<'this> {
    pub fn new(arena: &'this mut Arena, loader: &'this mut FileLoader, ast: ASTTree) -> Self {
        let src = loader.load(ast.id);
        Self {
            ast,
            arena,
            loader,
            src,
        }
    }

    pub fn generate<'a>(&mut self) -> ModuleScope<'a> {
        let mut out = ModuleScope::default();
        loop {
            let next = self.ast.next_node();
            match next {
                Some(v) => match v {
                    ASTNode::EOF => {
                        break;
                    }
                    _ => self.check_node(v.clone(), &mut out),
                },
                None => break,
            }
        }
        out
    }

    pub(crate) fn check_node(&mut self, input: ASTNode, scope: &mut ModuleScope) {
        match input {
            ASTNode::FunctionDeclaration {
                name,
                params,
                block: _,
                generics: _,
                out_type,
            } => self.parse_function_declaration(name, params, out_type, scope),
            ASTNode::ViewportDeclaration {
                name,
                params,
                block: _,
            } => self.parse_viewport_declaration(name, params, scope),
            ASTNode::ModStatement { name, tree } => {
                self.parse_mod_statement(name, tree, scope);
            }
            _ => {}
        }
    }

    pub(crate) fn parse_function_declaration(
        &mut self,
        name: Span,
        params: FunctionDeclarationParameters,
        out_type: Option<Path>,
        scope: &mut ModuleScope,
    ) {
        let return_type = match out_type {
            None => Type::Nothing,
            Some(v) => self.type_from_path(&v),
        };
        let symbol = Symbol::Function {
            params,
            return_type,
        };
        let name = name.parse_arena(self.src, self.arena);
        scope.symbols.insert(name, symbol);
    }

    pub(crate) fn parse_viewport_declaration(
        &mut self,
        name: Span,
        params: FunctionDeclarationParameters,
        scope: &mut ModuleScope,
    ) {
        let symbol = Symbol::Viewport { params };
        let name = name.parse_arena(self.src, self.arena);
        scope.symbols.insert(name, symbol);
    }

    pub(crate) fn parse_mod_statement(
        &mut self,
        name: Span,
        mut tree: ASTTree,
        parent: &mut ModuleScope,
    ) {
        let old = self.src;
        self.src = self.loader.load(tree.id);
        let mod_name = name.parse_arena(old, self.arena);
        let mut mod_scope = ModuleScope::default();
        loop {
            let next = tree.next_node();
            match next {
                None => break,
                Some(v) => {
                    self.check_node(v, &mut mod_scope);
                }
            }
        }
        parent.submodules.insert(mod_name, mod_scope);
        self.src = old;
    }

    pub(crate) fn type_from_path(&mut self, input: &Path) -> Type {
        let vec = input.0.clone();
        use Type::*;

        if vec.len() == 1 {
            let first = vec.first();
            match first {
                None => {}
                Some(PathNode::Named { name, param: _ }) => {
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

    pub(crate) fn path_to_symbol_name(&mut self, input: &Path) -> SymbolName {
        let vec = input.0.clone();
        let mut out = SymbolName::new();
        let mut iter = vec.iter();
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

#[derive(Hash, Clone, Debug)]
pub enum Symbol {
    Function {
        return_type: Type,
        params: FunctionDeclarationParameters,
    },
    Viewport {
        params: FunctionDeclarationParameters,
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
