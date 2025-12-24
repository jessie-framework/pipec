#![allow(dead_code)]

use pipec_ast::ast::{ASTNode, FunctionDeclarationParameters, Path, PathNode, asttree::ASTTree};
use std::collections::HashMap;
pub struct SemanticAnalyzer {
    stream: ASTTree,
    imports: ImportTree,
}

#[derive(Default)]
pub struct ImportTree(HashMap<Path, Symbol>);

/// A table to check the type of all symbols in the source code.
/// It would look like this if flattened out :
/// main : function() -> void;
/// rect : component(fport,fport,fport,fport,std::graphics::color);
/// math_utils::add_two : function(i32,i32) -> void;
/// etc etc
#[derive(Debug, Default)]
pub struct GlobalSymbolTree {
    symbols: HashMap<Path, Symbol>,
}

impl GlobalSymbolTree {
    pub fn gen_symbols(&mut self, tree: &mut ASTTree) {
        tree.reset();
        let path = Path::default();
        loop {
            let next = tree.next_node();
            match next {
                Some(v) => match v {
                    ASTNode::FunctionDeclaration {
                        name,
                        params,
                        block: _,
                        out_type,
                    } => self.add_function_declaration(name, params, out_type, path.clone()),
                    ASTNode::ViewportDeclaration {
                        name,
                        params,
                        block: _,
                    } => self.add_viewport_declaration(name, params, path.clone()),
                    ASTNode::ModStatement { name, tree } => {
                        self.add_mod_statement(name, tree, path.clone())
                    }
                    _ => {}
                },
                None => break,
            }
        }
    }

    #[inline]
    pub(crate) fn add_mod_statement(&mut self, name: &String, tree: &ASTTree, mut path: Path) {
        let stream = tree.stream();

        path.add_child(PathNode {
            name: name.to_string(),
            param: None,
        });
        let path = path.clone();
        for i in stream {
            match i {
                ASTNode::FunctionDeclaration {
                    name,
                    params,
                    block: _,
                    out_type,
                } => self.add_function_declaration(name, params, out_type, path.clone()),
                ASTNode::ViewportDeclaration {
                    name,
                    params,
                    block: _,
                } => self.add_viewport_declaration(name, params, path.clone()),
                ASTNode::ModStatement { name, tree } => {
                    self.add_mod_statement(name, tree, path.clone())
                }
                _ => {}
            }
        }
    }

    #[inline]
    pub(crate) fn add_viewport_declaration(
        &mut self,
        name: &String,
        params: &FunctionDeclarationParameters,
        path: Path,
    ) {
        let mut path = path.clone();
        path.add_child(PathNode {
            name: name.to_string(),
            param: None,
        });
        self.symbols.insert(
            path,
            Symbol::Viewport {
                params: Type::from_path_params(params),
            },
        );
    }

    #[inline]
    pub(crate) fn add_function_declaration(
        &mut self,
        name: &String,
        params: &FunctionDeclarationParameters,
        out_type: &Option<Path>,
        path: Path,
    ) {
        let mut path = path.clone();
        path.add_child(PathNode {
            name: name.to_string(),
            param: None,
        });
        self.symbols.insert(
            path,
            Symbol::Function {
                function_type: Type::from_path(out_type),
                params: Type::from_path_params(params),
            },
        );
    }
}

#[derive(Hash, Debug)]
pub enum Symbol {
    Function {
        function_type: Type,
        params: Vec<Type>,
    },
    Viewport {
        params: Vec<Type>,
    },
}

#[derive(Hash, Debug)]
pub enum Type {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    Void,
    Custom(Path),
    RenderSide(Box<Self>),
}

impl Type {
    pub fn from_path(input: &Option<Path>) -> Self {
        if let Some(v) = input {
            if v.0.len() == 1
                && let PathNode { name, param } = &v.0[0]
                && param.is_none()
            {
                match name.as_str() {
                    "i8" => return Self::I8,
                    "u8" => return Self::U8,
                    "i16" => return Self::I16,
                    "u16" => return Self::U16,
                    "i32" => return Self::I32,
                    "u32" => return Self::U32,
                    "i64" => return Self::I64,
                    "u64" => return Self::U64,
                    "void" => return Self::Void,
                    _ => {}
                }
            }
            return Self::Custom(input.clone().unwrap().clone());
        }
        Self::Void
    }

    pub fn from_path_params(input: &FunctionDeclarationParameters) -> Vec<Self> {
        let mut out = vec![];
        for i in input.handle() {
            out.push(Type::from_path(&Some(i.arg_type.clone())));
        }

        out
    }
}

impl SemanticAnalyzer {
    pub fn new(tree: ASTTree) -> Self {
        Self {
            stream: tree,
            imports: ImportTree::default(),
        }
    }
}
