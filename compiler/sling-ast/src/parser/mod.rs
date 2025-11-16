use crate::tokenizer::Token;
use crate::tokenizer::tokentree::TokenTree;

pub struct Parser {
    #[allow(dead_code)]
    tokens: TokenTree,
}

impl Parser {
    #[inline]
    pub(crate) fn advance_stream(&mut self) -> Option<&Token> {
        self.tokens.next_token()
    }
    #[inline]
    pub(crate) fn peek_stream(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    pub fn parse_value(&mut self) -> Parsed {
        self.consume_whitespace();
        match self.tokens.peek() {
            Some(v) => match v {
                Token::UsingKeyword => self.consume_using_keyword(),
                Token::MainKeyword => self.consume_main_keyword(),
                _ => todo!(),
            },
            None => Parsed::EOF,
        }
    }

    #[inline]
    pub(crate) fn consume_main_keyword(&mut self) -> Parsed {
        self.advance_stream();
        self.consume_whitespace();
        Parsed::MainFunction {
            block: self.consume_function_block(),
        }
    }

    #[inline]
    pub(crate) fn consume_function_block(&mut self) -> Block {
        if self.advance_stream() != Some(&Token::LeftCurly) {
            //TODO : compiler error because left curly bracket was expected
        }
        let mut block = Block::default();
        loop {
            self.consume_whitespace();
            let next = self.peek_stream();
            if next == Some(&Token::RightCurly) {
                break;
            }
            block.push(self.consume_a_block_statement());
        }
        block
    }

    #[inline]
    pub(crate) fn consume_a_block_statement(&mut self) -> FunctionBlockStatements {
        loop {
            match self.advance_stream() {
                Some(v) => match v {
                    Token::Whitespace => continue,
                    Token::LetKeyword => return self.consume_variable_declaration(),
                    _ => {
                        //TODO : compiler error
                        unreachable!();
                    }
                },
                None => {
                    // TODO : compiler error
                    unreachable!()
                }
            }
        }
    }

    #[inline]
    pub(crate) fn consume_variable_declaration(&mut self) -> FunctionBlockStatements {
        // let x : u32 = 0;
        self.consume_whitespace();
        let varname: String;
        let vartype: Node;
        let declexpr: Expression;
        match self.advance_stream() {
            Some(Token::Ident(variable_name)) => {
                varname = variable_name.to_string();
            }
            _ => {
                //TODO: compiler error
                unreachable!()
            }
        }
        self.consume_whitespace();
        match self.advance_stream() {
            Some(Token::Colon) => {
                self.consume_whitespace();
                vartype = self.consume_a_node();
                self.consume_whitespace();
                match self.advance_stream() {
                    Some(Token::EqualSign) => {
                        declexpr = self.consume_an_expression();
                    }
                    _anything_else => {
                        //TODO : compiler error
                        unreachable!();
                    }
                }
            }

            Some(Token::EqualSign) => {
                self.consume_whitespace();
                declexpr = self.consume_an_expression();
                vartype = declexpr.expressiontype.clone();
            }

            _ => {
                //TODO : compiler error
                unreachable!()
            }
        }
        self.consume_whitespace();
        self.consume_a_semicolon();

        FunctionBlockStatements::VariableDeclaration {
            variablename: varname,
            variabletype: vartype,
            declarationexpression: declexpr,
        }
    }

    #[inline]
    pub(crate) fn consume_a_semicolon(&mut self) {
        if self.advance_stream() == Some(&Token::Semicolon) {
            return;
        }
        //TODO : compiler error
        todo!();
    }

    #[inline]
    pub(crate) fn consume_an_expression(&mut self) -> Expression {
        todo!()
    }

    #[inline]
    pub(crate) fn consume_using_keyword(&mut self) -> Parsed {
        self.advance_stream();
        self.consume_whitespace();

        Parsed::UsingStatement {
            using: self.consume_a_node(),
        }
    }

    #[inline]
    fn consume_a_node(&mut self) -> Node {
        let mut out = Node::default();
        loop {
            match self.advance_stream() {
                Some(Token::Ident(ident_value)) => {
                    out.add_child(NodeName::Named(ident_value.clone()));
                    if self.advance_stream() == Some(&Token::DoubleColon) {
                        continue;
                    } else {
                        //TODO : compiler error because two colons were expected
                        unreachable!();
                    }
                }
                Some(Token::Semicolon) => {
                    break;
                }
                _ => {
                    // TODO : compiler error because an ident token was expected
                    unreachable!();
                }
            }
        }
        out
    }

    #[inline]
    pub(crate) fn consume_whitespace(&mut self) {
        while self.tokens.peek() == Some(&Token::Whitespace) {
            self.tokens.next_token();
        }
    }
}
#[derive(Clone)]
pub struct Node(Vec<NodeName>);

impl Default for Node {
    fn default() -> Self {
        Self(vec![NodeName::Root])
    }
}
impl Node {
    pub fn add_child(&mut self, input: NodeName) {
        self.0.push(input);
    }
}

#[derive(Clone)]
pub enum NodeName {
    Root,
    Named(String),
}

pub enum Parsed {
    MainFunction { block: Block },
    FunctionBlock { contents: Block },
    StaticVariableDeclaration,                                 // TODO
    ComponentDeclaration { block: ComponentDeclarationBlock }, // TODO
    UsingStatement { using: Node },
    EOF,
}

#[allow(dead_code)]
pub struct ComponentDeclarationBlock {
    contents: Vec<ComponentDeclarationBlockStatements>,
}

pub enum ComponentDeclarationBlockStatements {
    FinalVariableDeclaration,
    ConstVariableDeclaration,
    FunctionDeclaration { block: Block },
    RenderBlockDeclaration { block: RenderBlock },
}

#[allow(dead_code)]
pub struct RenderBlock {
    vertices_block: VerticesBlock,
    fragments_block: FragmentsBlock,
}

pub struct VerticesBlock {}
pub struct FragmentsBlock {}

pub enum FunctionBlockStatements {
    VariableDeclaration {
        variablename: String,
        variabletype: Node,
        declarationexpression: Expression,
    },
    ViewportBlock {
        width: Expression,
        height: Expression,
        block: Block,
    },
}

#[derive(Clone)]
pub struct Expression {
    expressiontype: Node,
}

pub enum VariableType {
    Const,
    Final,
}

pub struct Block(Vec<FunctionBlockStatements>);
impl Block {
    pub fn push(&mut self, input: FunctionBlockStatements) {
        self.0.push(input);
    }
}

impl Default for Block {
    fn default() -> Self {
        Self(Vec::with_capacity(30))
    }
}
