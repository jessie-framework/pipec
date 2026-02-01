use pipec_arena::AStr;
use pipec_arena::{ASlice, Arena};
use pipec_ast::ast::FunctionDeclarationParameters;
use pipec_ast::ast::Path;
use pipec_ast::ast::PathNode;
use pipec_ast::ast::asttree::ASTTree;
use pipec_ast::ast::{ASTNode, Block, Generics};
use pipec_file_loader::FileLoader;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct GlobalSymbolTree<'this> {
    ast: ASTTree,
    loader: &'this mut FileLoader,
    arena: &'this mut Arena,
    src: ASlice<AStr>,
    attribute_cache: HashSet<LanguageAttribute>,
}

#[derive(Default, Debug)]
pub struct ModuleScope<'a> {
    pub symbols: HashMap<&'a str, Symbol<'a>>,
    pub submodules: HashMap<&'a str, Self>,
}

impl<'this> GlobalSymbolTree<'this> {
    pub fn new(arena: &'this mut Arena, loader: &'this mut FileLoader, ast: ASTTree) -> Self {
        let src = loader.load(ast.id);
        let attribute_cache = HashSet::new();
        Self {
            ast,
            arena,
            loader,
            src,
            attribute_cache,
        }
    }

    pub fn generate<'a>(&mut self) -> ModuleScope<'a> {
        let mut out = ModuleScope::default();
        let stream = self.ast.stream.clone();
        let mut iter = stream.iter();
        loop {
            let next = iter.next();
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
        println!("{:#?}", &out);
        self.import_using(&mut out);
        out
    }

    pub(crate) fn check_node(&mut self, input: ASTNode, scope: &mut ModuleScope) {
        match input {
            ASTNode::FunctionDeclaration {
                name,
                params,
                block,
                generics,
                out_type,
            } => {
                let parsed_name = name.parse_arena(self.src, self.arena);
                println!("found function {parsed_name}");
                scope.symbols.insert(
                    parsed_name,
                    Symbol::Function {
                        out_type,
                        params,
                        block,
                        generics,
                    },
                );
            }
            ASTNode::ViewportDeclaration {
                name,
                params,
                block,
            } => {
                let parsed_name = name.parse_arena(self.src, self.arena);
                scope
                    .symbols
                    .insert(parsed_name, Symbol::Viewport { params, block });
            }
            ASTNode::ModStatement { name, tree } => {
                println!("consuming mod");
                let old = self.src;
                self.src = self.loader.load(tree.id);
                let mod_name = name.parse_arena(old, self.arena);
                let mut mod_scope = ModuleScope::default();
                let stream = tree.stream.clone();
                let mut iter = stream.iter();
                loop {
                    let next = iter.next();
                    match next {
                        Some(v) => match v {
                            ASTNode::EOF => {
                                break;
                            }
                            _ => self.check_node(v.clone(), &mut mod_scope),
                        },
                        None => break,
                    }
                }
                scope.submodules.insert(mod_name, mod_scope);
                self.src = old;
            }
            _ => {}
        }
    }

    pub(crate) fn import_using(&mut self, scope: &mut ModuleScope) {
        let stream = self.ast.stream.clone();
        let iter = stream.iter();
        for next in iter {
            match next {
                ASTNode::EOF => break,
                ASTNode::UsingStatement { using } => self.use_path(using, scope),
                ASTNode::ModStatement { name, tree } => {
                    println!("importing module appearently");
                    let old = self.src;
                    self.src = self.loader.load(tree.id);
                    let mod_name = name.parse_arena(old, self.arena);
                    let mod_scope = scope.submodules.get_mut(mod_name).unwrap();
                    let stream = tree.stream.clone();
                    for item in stream {
                        match item {
                            ASTNode::EOF => break,
                            ASTNode::UsingStatement { using } => self.use_path(&using, mod_scope),
                            _ => {}
                        }
                    }
                    self.src = old;
                }
                _ => {}
            }
        }
    }

    #[inline]
    pub(crate) fn use_path(&mut self, input: &Path, target: &mut ModuleScope) {
        println!("using {input:#?}");
        let mut iter = input.0.iter().peekable();
        let mut current: &mut ModuleScope = target;
        loop {
            let next = iter.next();
            if iter.peek().is_none() {
                match next {
                    Some(PathNode::Singly { name, generics }) => {
                        if !generics.0.is_empty() {
                            // TODO : compiler error
                            unreachable!();
                        }
                        let parsed_name = name.parse_arena(self.src, self.arena);
                        let current_ptr: *const ModuleScope = current;
                        target
                            .symbols
                            .insert(parsed_name, Symbol::Alias(current_ptr));
                    }
                    Some(PathNode::Multi(paths)) => {
                        for path in paths {
                            let inner = path.0.clone();
                            let mut new_path_inner = input.0.clone();
                            new_path_inner.extend_from_slice(&inner);

                            let new_path = Path(new_path_inner);
                            self.use_path(&new_path, target);
                        }
                    }
                    None => unreachable!("path is empty"),
                }
                break;
            } else if let Some(PathNode::Singly { name, generics }) = next {
                if !generics.0.is_empty() {
                    // TODO : compiler error
                    unreachable!();
                }
                let module_name = name.parse_arena(self.src, self.arena);
                println!("{module_name} is the modules name");
                current = current.submodules.get_mut(module_name).unwrap_or_else(|| {
                    // TODO : compiler error
                    unreachable!();
                });
            }
        }
    }
}

#[derive(Hash, Clone, Debug)]
pub enum Symbol<'a> {
    Function {
        out_type: Path,
        params: FunctionDeclarationParameters,
        block: Block,
        generics: Generics,
    },
    Viewport {
        params: FunctionDeclarationParameters,
        block: Block,
    },

    Builtin(LanguageAttribute),
    Alias(*const ModuleScope<'a>),
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub enum LanguageAttribute {
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
}
