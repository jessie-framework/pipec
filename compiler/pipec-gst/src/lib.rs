use pipec_arena::Arena;
use pipec_arena_structures::ListNode;
use pipec_ast::ast::ASTNode;
use pipec_ast::ast::FunctionDeclarationParameters;
use pipec_ast::ast::Path;
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
    src: String,
}

impl<'this> GlobalSymbolTree<'this> {
    pub fn new(arena: &'this mut Arena, loader: &'this mut FileLoader, ast: ASTTree) -> Self {
        let map = HashMap::new();
        let src = loader.load(ast.id);
        let context = SymbolName::default();
        Self {
            ast,
            arena,
            loader,
            map,
            src: src.to_string(),
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
            _ => todo!(),
        }
    }

    pub(crate) fn parse_function_declaration(
        &mut self,
        name: Span,
        params: FunctionDeclarationParameters,
        out_type: Option<Path>,
    ) {
        let return_type = match out_type {
            None => Type::Void,
            Some(v) => self.type_from_path(&v),
        };
        let params = self.ast_to_gst_params(params);
        let symbol = Symbol::Function {
            params,
            return_type,
        };
        let mut cloned = self.context.clone();
        let name = name.parse(&self.src).to_string();
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
        let name = name.parse(&self.src).to_string();
        cloned.path.push(name);

        self.map.insert(cloned, symbol);
    }

    pub(crate) fn parse_mod_statement(&mut self, name: Span, mut tree: ASTTree) {
        let name = name.parse(&self.src).to_string();
        let old_src = self.src.clone();
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
                ListNode::Node(val, _) => {
                    let name = val.name.parse(&self.src);
                    match name {
                        "i8" => return I8,
                        "u8" => return U8,
                        "f8" => return F8,
                        "i16" => return I16,
                        "u16" => return U16,
                        "f16" => return F16,
                        "i32" => return I32,
                        "u32" => return U32,
                        "f32" => return F32,
                        "i64" => return I64,
                        "u64" => return U64,
                        "f64" => return F64,
                        "fport" => return FPort,
                        "void" => return Void,
                        _ => {}
                    }
                }
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
            let name = i.name.parse(&self.src).to_string();
            let p_type = self.type_from_path(&i.arg_type);
            out.0.push((name, p_type));
        }
        out
    }

    pub(crate) fn path_to_symbol_name(&mut self, input: &Path) -> SymbolName {
        let vec = input.0.clone();
        let mut out = SymbolName::default();
        for i in vec.iter(self.arena) {
            let name = i.name.parse(&self.src).to_string();
            out.path.push(name);
        }
        out
    }
}

#[derive(Hash, Clone, PartialEq, Eq, Default, Debug)]
pub struct SymbolName {
    pub(crate) path: Vec<String>,
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
    I8,
    U8,
    F8,
    I16,
    U16,
    F16,
    I32,
    U32,
    F32,
    I64,
    U64,
    F64,
    FPort,
    Void,
    Link(SymbolName),
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct FunctionParameters(Vec<(String, Type)>);
