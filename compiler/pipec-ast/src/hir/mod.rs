use std::path::PathBuf;
use std::sync::Arc;

use self::hirtree::HIRTree;
use crate::ASTFileReader;
use crate::RecursiveGuard;
use crate::tokenizer::DigitType;
use crate::tokenizer::Token;
use crate::tokenizer::tokentree::TokenTree;
use pipec_cache::{Cached, Decode, Encode};

pub mod hirtree;

pub struct HIRGenerator<'this> {
    tokens: &'this mut TokenTree,
    guard: &'this mut RecursiveGuard,
    path: PathBuf,
    cache_dir: Arc<Option<PathBuf>>,
}

impl<'this> HIRGenerator<'this> {
    pub fn tree(mut self) -> HIRTree {
        let mut out = Vec::with_capacity(30);
        loop {
            let next = self.parse_value();
            if next == HIRNode::EOF {
                break;
            }
            out.push(next);
        }
        HIRTree::new(out)
    }

    pub fn new(
        tokens: &'this mut TokenTree,
        path: PathBuf,
        guard: &'this mut RecursiveGuard,
        cache_dir: Arc<Option<PathBuf>>,
    ) -> Self {
        let path = path.parent().unwrap().to_path_buf();
        Self {
            tokens,
            path,
            guard,
            cache_dir,
        }
    }

    #[inline]
    pub(crate) fn advance_stream(&mut self) -> Option<&Token> {
        println!("{:#?}", self.peek_stream());
        self.tokens.next_token()
    }
    #[inline]
    pub(crate) fn peek_stream(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    pub fn parse_value(&mut self) -> HIRNode {
        self.consume_whitespace();
        match self.peek_stream() {
            Some(v) => match v {
                Token::UsingKeyword => self.consume_using_keyword(),
                Token::ModuleKeyword => self.consume_module_keyword(),
                Token::ComponentKeyword => self.consume_component_keyword(),
                Token::ViewportKeyword => self.consume_viewport_keyword(),
                Token::FunctionKeyword => self.consume_function_keyword(),
                _v => {
                    todo!();
                }
            },
            None => HIRNode::EOF,
        }
    }

    #[inline]
    pub(crate) fn consume_function_keyword(&mut self) -> HIRNode {
        self.advance_stream();
        self.consume_whitespace();
        let name = self.must_ident();
        self.consume_whitespace();
        let params = self.consume_function_parameters();
        self.consume_whitespace();
        let mut out_type = None;
        if self.next_is(Token::ThinArrow) {
            self.advance_stream();
            self.consume_whitespace();
            out_type = Some(self.consume_a_path());
        }
        self.consume_whitespace();
        let block = self.consume_function_block();

        HIRNode::FunctionDeclaration {
            name,
            params,
            block,
            out_type,
        }
    }

    #[inline]
    pub(crate) fn consume_viewport_keyword(&mut self) -> HIRNode {
        self.advance_stream();
        self.consume_whitespace();
        let name = self.must_ident();
        self.consume_whitespace();
        let params = self.consume_function_parameters();
        self.consume_whitespace();
        let block = self.consume_function_block();
        HIRNode::ViewportDeclaration {
            name,
            params,
            block,
        }
    }

    #[inline]
    pub(crate) fn next_is(&mut self, next: Token) -> bool {
        self.peek_stream() == Some(&next)
    }

    #[inline]
    pub(crate) fn consume_function_parameters(&mut self) -> FunctionDeclarationParameters {
        let mut out = FunctionDeclarationParameters::new();
        self.must(Token::LeftParenthesis);
        loop {
            self.consume_whitespace();
            if self.next_is(Token::RightParenthesis) {
                self.advance_stream();
                break;
            }
            out.push(self.consume_function_parameter());
            self.consume_whitespace();
            if self.next_is(Token::Comma) {
                self.advance_stream();
                continue;
            }
        }
        out
    }

    #[inline]
    pub(crate) fn must_ident(&mut self) -> String {
        if let Some(Token::Ident(v)) = self.advance_stream() {
            return v.to_string();
        }
        unreachable!()
    }

    #[inline]
    pub(crate) fn consume_function_parameter(&mut self) -> FunctionDeclarationParameter {
        let name = self.must_ident();
        self.consume_whitespace();
        self.must(Token::Colon);
        self.consume_whitespace();
        let arg_type = self.consume_a_path();

        FunctionDeclarationParameter { name, arg_type }
    }

    #[inline]
    pub(crate) fn must(&mut self, val: Token) {
        if self.advance_stream() != Some(&val) {
            // TODO : compiler error
            unreachable!()
        }
    }

    #[inline]
    pub(crate) fn consume_module_keyword(&mut self) -> HIRNode {
        self.advance_stream();
        self.consume_whitespace();
        let mod_path = match self.advance_stream() {
            Some(Token::Ident(v)) => v.to_owned(),
            _ => {
                //todo : compiler error
                unreachable!()
            }
        };
        self.consume_whitespace();
        match self.peek_stream() {
            Some(Token::Semicolon) => self.consume_node_from_fs(mod_path),
            Some(Token::LeftCurly) => self.consume_mod_block(mod_path),
            _ => {
                //TODO : compiler error
                unreachable!()
            }
        }
    }

    #[inline]
    pub(crate) fn consume_mod_block(&mut self, mod_path: String) -> HIRNode {
        self.advance_stream();
        let mut nodes = Vec::with_capacity(10);
        loop {
            self.consume_whitespace();
            if self.peek_stream() == Some(&Token::RightCurly) {
                self.advance_stream();
                break;
            }
            nodes.push(self.parse_value());
        }
        HIRNode::ModStatement {
            name: mod_path,
            tree: HIRTree::from_vec(nodes),
        }
    }

    #[inline]
    pub(crate) fn consume_node_from_fs(&mut self, mod_path: String) -> HIRNode {
        self.advance_stream();
        let path1 = {
            let mut cloned = self.path.clone();
            cloned.push(format!("{}/mod.pipec", &mod_path));
            cloned
        };
        let path2 = {
            let mut cloned = self.path.clone();
            cloned.push(format!("{}.pipec", &mod_path));
            cloned
        };

        if self.guard.contains(&path1) || self.guard.contains(&path2) {
            //TODO : compiler error
            unreachable!();
        }
        self.guard.push(path1.clone());
        self.guard.push(path2.clone());

        if path1.exists() && path2.exists() {
            // TODO : compiler error
            unreachable!();
        }
        if path1.exists() {
            let (mut reader, link) = ASTFileReader::new(&path1, self.cache_dir.clone())
                .unwrap_or_else(|_| {
                    // TODO : compiler error
                    unreachable!();
                });
            let hir = reader.generate_hir(self.guard, self.cache_dir.clone());
            reader.upload_to_cache(link, self.cache_dir.clone());
            return HIRNode::ModStatement {
                name: mod_path,
                tree: hir,
            };
        }

        if path2.exists() {
            let (mut reader, link) = ASTFileReader::new(&path2, self.cache_dir.clone())
                .unwrap_or_else(|_| {
                    // TODO : compiler error
                    unreachable!();
                });
            let hir = reader.generate_hir(self.guard, self.cache_dir.clone());
            reader.upload_to_cache(link, self.cache_dir.clone());
            return HIRNode::ModStatement {
                name: mod_path,
                tree: hir,
            };
        }

        println!("{path1:#?},{path2:#?}");
        unreachable!()
    }

    #[inline]
    pub(crate) fn consume_component_keyword(&mut self) -> HIRNode {
        self.advance_stream();
        self.consume_whitespace();
        if let Some(Token::Ident(v)) = self.advance_stream() {
            return HIRNode::ComponentDeclaration {
                name: v.to_string(),
                block: self.consume_component_declaration_block(),
            };
        }
        //TODO : compiler error
        unreachable!();
    }

    #[inline]
    pub(crate) fn consume_component_declaration_block(&mut self) -> ComponentDeclarationBlock {
        self.consume_whitespace();
        match self.advance_stream() {
            Some(Token::LeftCurly) => {}
            _v => {
                //TODO : compiler error
                unreachable!();
            }
        }
        let mut contents = Vec::with_capacity(30);

        loop {
            self.consume_whitespace();
            let next = self.peek_stream();
            if next == Some(&Token::RightCurly) {
                self.advance_stream();
                break;
            }
            contents.push(self.consume_component_declaration_statement());
        }
        ComponentDeclarationBlock { contents }
    }

    #[inline]
    pub(crate) fn consume_component_declaration_statement(
        &mut self,
    ) -> ComponentDeclarationBlockStatements {
        match self.advance_stream() {
            Some(Token::FinalKeyword) => self.consume_final_variable_declaration(),
            Some(Token::RenderKeyword) => self.consume_render_block(),
            Some(Token::PublicKeyword) => self.consume_public_constructor(),
            _v => {
                //TODO : compiler error
                unreachable!();
            }
        }
    }

    #[inline]
    pub(crate) fn consume_public_constructor(&mut self) -> ComponentDeclarationBlockStatements {
        self.consume_whitespace();
        let expression = self.consume_an_expression();
        self.consume_whitespace();
        self.consume_a_semicolon();
        ComponentDeclarationBlockStatements::PublicConstructor { expression }
    }

    #[inline]
    pub(crate) fn consume_render_block(&mut self) -> ComponentDeclarationBlockStatements {
        self.consume_whitespace();
        let block = self.consume_render_block_inner();
        ComponentDeclarationBlockStatements::RenderBlockDeclaration { block }
    }

    #[inline]
    pub(crate) fn consume_render_block_inner(&mut self) -> RenderBlock {
        if self.advance_stream() != Some(&Token::LeftCurly) {
            //TODO : compiler error
            unreachable!();
        }
        self.consume_whitespace();
        let vertices_block = self.consume_vertices_block();
        self.consume_whitespace();
        let fragments_block = self.consume_fragments_block();
        self.consume_whitespace();
        if self.advance_stream() != Some(&Token::RightCurly) {
            //TODO : compiler error
            unreachable!();
        }
        RenderBlock {
            vertices_block,
            fragments_block,
        }
    }

    #[inline]
    pub(crate) fn consume_vertices_block(&mut self) -> VerticesBlock {
        {
            let next = self.advance_stream();
            if next != Some(&Token::VerticesKeyword) {
                // TODO : compiler error
                unreachable!()
            }
        }
        self.consume_whitespace();
        VerticesBlock {
            block: self.consume_function_block(),
        }
    }

    #[inline]
    pub(crate) fn consume_fragments_block(&mut self) -> FragmentsBlock {
        {
            let next = self.advance_stream();
            if next != Some(&Token::FragmentsKeyword) {
                // TODO : compiler error
                unreachable!()
            }
        }
        self.consume_whitespace();
        FragmentsBlock {
            block: self.consume_function_block(),
        }
    }

    #[inline]
    pub(crate) fn consume_final_variable_declaration(
        &mut self,
    ) -> ComponentDeclarationBlockStatements {
        self.consume_whitespace();
        let variablename: String = if let Some(Token::Ident(v)) = self.advance_stream() {
            v.to_string()
        } else {
            //TODO : compiler error
            unreachable!();
        };
        self.consume_whitespace();
        let variabletype: Option<Path>;
        let declarationexpression: Option<Expression>;
        match self.advance_stream() {
            Some(Token::Colon) => {
                self.consume_whitespace();
                variabletype = Some(self.consume_a_path());
                self.consume_whitespace();
                if let Some(Token::EqualSign) = self.peek_stream() {
                    declarationexpression = Some(self.consume_an_expression());
                } else {
                    declarationexpression = None;
                }
            }
            Some(Token::EqualSign) => {
                self.consume_whitespace();
                variabletype = None;
                declarationexpression = Some(self.consume_an_expression());
            }
            _v => {
                //TODO : compile error
                unreachable!();
            }
        }
        self.consume_whitespace();
        self.consume_a_semicolon();
        ComponentDeclarationBlockStatements::FinalVariableDeclaration {
            variablename,
            variabletype,
            declarationexpression,
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
                self.advance_stream();
                break;
            }
            block.push(self.consume_a_block_statement());
        }
        block
    }

    #[inline]
    pub(crate) fn consume_a_block_statement(&mut self) -> FunctionBlockStatements {
        self.consume_whitespace();
        match self.peek_stream() {
            Some(v) => match v {
                Token::LetKeyword => self.consume_variable_declaration(),
                Token::ExportKeyword => self.consume_export_declaration(),
                _ => self.consume_expression_statement(),
            },
            None => {
                // TODO : compiler error
                unreachable!()
            }
        }
    }

    #[inline]
    pub(crate) fn consume_expression_statement(&mut self) -> FunctionBlockStatements {
        let expression = self.consume_an_expression();
        self.consume_whitespace();
        self.consume_a_semicolon();
        FunctionBlockStatements::ExpressionStatement { expression }
    }

    #[inline]
    pub(crate) fn consume_export_declaration(&mut self) -> FunctionBlockStatements {
        self.advance_stream();
        self.consume_whitespace();
        let exporting: Exported = match self.advance_stream() {
            Some(Token::Hash) => match self.advance_stream() {
                Some(Token::Ident(name)) => match name.as_str() {
                    "col" => Exported::ColorBuiltin,
                    "pos" => Exported::PositionBuiltin,
                    _ => {
                        // TODO : compiler error
                        unreachable!()
                    }
                },
                _ => {
                    // TODO : compiler error
                    unreachable!()
                }
            },
            Some(Token::Ident(name)) => Exported::Custom(name.to_string()),
            _ => {
                //TODO : compiler error
                unreachable!()
            }
        };
        let decl_type: Option<Path>;
        let decl_expr: Expression;
        self.consume_whitespace();
        match self.advance_stream() {
            Some(Token::EqualSign) => {
                self.consume_whitespace();
                decl_type = None;
                decl_expr = self.consume_an_expression();
                self.consume_whitespace();
                self.consume_a_semicolon();
                FunctionBlockStatements::ExportDeclaration {
                    exporting,
                    exporttype: decl_type,
                    expression: decl_expr,
                }
            }
            Some(Token::Colon) => {
                self.consume_whitespace();
                decl_type = Some(self.consume_a_path());
                self.consume_whitespace();
                if self.advance_stream() == Some(&Token::EqualSign) {
                    self.consume_whitespace();
                    decl_expr = self.consume_an_expression();
                    self.consume_whitespace();
                    self.consume_a_semicolon();
                    return FunctionBlockStatements::ExportDeclaration {
                        exporting,
                        exporttype: decl_type,
                        expression: decl_expr,
                    };
                }
                // TODO : compiler error
                unreachable!();
            }
            _ => {
                //TODO : compiler error
                unreachable!();
            }
        }
    }

    #[inline]
    pub(crate) fn consume_variable_declaration(&mut self) -> FunctionBlockStatements {
        self.advance_stream();
        // let x : u32 = 0;
        self.consume_whitespace();
        let varname: String;
        let vartype: Option<Path>;
        let declexpr: Option<Expression>;
        let mut is_mutable = false;
        match self.advance_stream() {
            Some(Token::Ident(variable_name)) => {
                varname = variable_name.to_string();
            }
            Some(Token::MutableKeyword) => {
                is_mutable = true;
                self.consume_whitespace();
                if let Some(Token::Ident(variable_name)) = self.advance_stream() {
                    varname = variable_name.to_string();
                } else {
                    //TODO : compiler error
                    unreachable!()
                }
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
                vartype = Some(self.consume_a_path());
                self.consume_whitespace();
                match self.advance_stream() {
                    Some(Token::EqualSign) => {
                        declexpr = Some(self.consume_an_expression());
                    }
                    _anything_else => {
                        //TODO : compiler error
                        unreachable!();
                    }
                }
            }

            Some(Token::EqualSign) => {
                self.consume_whitespace();
                declexpr = Some(self.consume_an_expression());
                vartype = None;
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
            is_mutable,
        }
        // TODO : update this function
    }

    #[inline]
    pub(crate) fn consume_a_semicolon(&mut self) {
        let peek = self.peek_stream();
        if peek == Some(&Token::Semicolon) {
            self.advance_stream();
            return;
        }
        //TODO : compiler error
        todo!();
    }

    #[inline]
    pub(crate) fn consume_an_expression(&mut self) -> Expression {
        self.consume_whitespace();
        match self.peek_stream() {
            Some(Token::Digit { .. }) => self.consume_number_expression(),
            Some(Token::String(_)) => self.consume_string_expression(),
            Some(Token::LeftParenthesis) => self.consume_tuple_expression(),
            Some(Token::LeftSquare) => self.consume_list_expression(),
            Some(Token::Tilde) => self.consume_tilde_expression(),
            Some(Token::Ident(_)) => self.consume_path_expression(),
            Some(Token::RequiredKeyword) => self.consume_required_expression(),

            _v => {
                //TODO : compiler error
                unreachable!();
            }
        }
    }

    #[inline]
    pub(crate) fn consume_required_expression(&mut self) -> Expression {
        self.advance_stream();
        self.consume_whitespace();
        Expression::RequiredExpression {
            value: Box::new(self.consume_an_expression()),
        }
    }

    #[inline]
    pub(crate) fn consume_path_expression(&mut self) -> Expression {
        self.consume_whitespace();
        let first = match self.peek_stream() {
            Some(Token::Ident(_)) => Expression::PathExpression {
                value: self.consume_a_path(),
            },
            _v => {
                //TODO : compile error
                unreachable!()
            }
        };
        self.consume_whitespace();
        match self.peek_stream() {
            Some(Token::Plus) => {
                self.advance_stream();
                Expression::UnaryOpExpression {
                    optype: UnaryOpType::Add,
                    lhs: Box::new(first),
                    rhs: Box::new(self.consume_an_expression()),
                }
            }
            Some(Token::Minus) => {
                self.advance_stream();
                Expression::UnaryOpExpression {
                    optype: UnaryOpType::Subtract,
                    lhs: Box::new(first),
                    rhs: Box::new(self.consume_an_expression()),
                }
            }
            Some(Token::Asterisk) => {
                self.advance_stream();
                Expression::UnaryOpExpression {
                    optype: UnaryOpType::Multiply,
                    lhs: Box::new(first),
                    rhs: Box::new(self.consume_an_expression()),
                }
            }
            Some(Token::Slash) => {
                self.advance_stream();
                Expression::UnaryOpExpression {
                    optype: UnaryOpType::Divide,
                    lhs: Box::new(first),
                    rhs: Box::new(self.consume_an_expression()),
                }
            }
            Some(Token::Modulo) => {
                self.advance_stream();
                Expression::UnaryOpExpression {
                    optype: UnaryOpType::Mod,
                    lhs: Box::new(first),
                    rhs: Box::new(self.consume_an_expression()),
                }
            }

            _ => first,
        }
    }
    #[inline]
    pub(crate) fn consume_list_expression(&mut self) -> Expression {
        self.advance_stream();
        let mut values = Vec::with_capacity(10);
        loop {
            values.push(self.consume_an_expression());
            self.consume_whitespace();

            let next = self.peek_stream();
            match next {
                Some(Token::Comma) => {
                    self.advance_stream();
                    continue;
                }
                Some(Token::RightSquare) => {
                    self.advance_stream();
                    return Expression::ListExpression { values };
                }
                _ => {
                    //TODO : compiler error
                    unreachable!();
                }
            }
        }
    }
    #[inline]
    pub(crate) fn consume_tilde_expression(&mut self) -> Expression {
        self.advance_stream();
        self.consume_whitespace();
        Expression::TildeExpression {
            value: Box::new(self.consume_an_expression()),
        }
    }

    #[inline]
    pub(crate) fn consume_string_expression(&mut self) -> Expression {
        if let Some(Token::Ident(v)) = self.advance_stream() {
            return Expression::StringExpression {
                value: v.to_string(),
            };
        }
        //TODO : compiler error
        unreachable!();
    }

    #[inline]
    pub(crate) fn consume_tuple_expression(&mut self) -> Expression {
        self.advance_stream();
        let mut values = Vec::with_capacity(10);
        loop {
            values.push(self.consume_an_expression());
            self.consume_whitespace();

            let next = self.peek_stream();
            match next {
                Some(Token::Comma) => {
                    self.advance_stream();
                    continue;
                }
                Some(Token::RightParenthesis) => {
                    self.advance_stream();
                    return Expression::TupleExpression { values };
                }
                _v => {
                    //TODO : compiler error
                    unreachable!();
                }
            }
        }
    }

    #[inline]
    pub(crate) fn consume_number_expression(&mut self) -> Expression {
        self.consume_whitespace();
        let first = match self.advance_stream() {
            Some(Token::Digit {
                val: value,
                digittype,
            }) => Expression::NumberExpression {
                value: value.to_string(),
                digittype: *digittype,
            },
            _v => {
                //TODO : compile error
                unreachable!()
            }
        };
        self.consume_whitespace();
        match self.peek_stream() {
            Some(Token::Plus) => {
                self.advance_stream();
                Expression::UnaryOpExpression {
                    optype: UnaryOpType::Add,
                    lhs: Box::new(first),
                    rhs: Box::new(self.consume_an_expression()),
                }
            }
            Some(Token::Minus) => {
                self.advance_stream();
                Expression::UnaryOpExpression {
                    optype: UnaryOpType::Subtract,
                    lhs: Box::new(first),
                    rhs: Box::new(self.consume_an_expression()),
                }
            }
            Some(Token::Asterisk) => {
                self.advance_stream();
                Expression::UnaryOpExpression {
                    optype: UnaryOpType::Multiply,
                    lhs: Box::new(first),
                    rhs: Box::new(self.consume_an_expression()),
                }
            }
            Some(Token::Slash) => {
                self.advance_stream();
                Expression::UnaryOpExpression {
                    optype: UnaryOpType::Divide,
                    lhs: Box::new(first),
                    rhs: Box::new(self.consume_an_expression()),
                }
            }
            Some(Token::Modulo) => {
                self.advance_stream();
                Expression::UnaryOpExpression {
                    optype: UnaryOpType::Mod,
                    lhs: Box::new(first),
                    rhs: Box::new(self.consume_an_expression()),
                }
            }

            _v => first,
        }
    }

    #[inline]
    pub(crate) fn consume_using_keyword(&mut self) -> HIRNode {
        self.advance_stream();
        self.consume_whitespace();

        let using = self.consume_a_path();
        self.consume_a_semicolon();
        HIRNode::UsingStatement { using }
    }

    #[inline]
    fn consume_a_path(&mut self) -> Path {
        let mut out = Path::default();
        loop {
            let next = self.peek_stream();
            match next {
                Some(Token::Ident(v)) => {
                    let name = v.to_string();
                    self.advance_stream();
                    let param = self.consume_path_param();
                    out.add_child(PathNode { name, param });
                    continue;
                }
                Some(Token::DoubleColon) => {
                    self.advance_stream();
                    continue;
                }
                _ => {
                    break;
                }
            }
        }
        out
    }

    #[inline]
    pub(crate) fn consume_path_param(&mut self) -> Option<FunctionNodeParams> {
        match self.peek_stream() {
            Some(Token::LeftParenthesis) => {
                if let Expression::TupleExpression { values } = self.consume_tuple_expression() {
                    return Some(FunctionNodeParams::Tuple(values));
                }
                // TODO : compiler error
                unreachable!();
            }
            Some(Token::LeftAngle) => Some(FunctionNodeParams::Angles(self.consume_angle_params())),
            _v => None,
        }
    }

    #[inline]
    pub(crate) fn consume_angle_params(&mut self) -> Vec<Path> {
        if self.advance_stream() != Some(&Token::LeftAngle) {
            //TODO : compiler error
        }
        let mut out = Vec::with_capacity(4);
        loop {
            self.consume_whitespace();
            match self.peek_stream() {
                Some(Token::Ident(_)) => {
                    out.push(self.consume_a_path());
                    self.consume_whitespace();
                    match self.peek_stream() {
                        Some(Token::Comma) => {
                            self.advance_stream();
                            continue;
                        }
                        Some(Token::Ident(_)) => {
                            continue;
                        }
                        Some(Token::RightAngle) => {
                            break;
                        }
                        _ => {
                            //TODO : compiler error
                            unreachable!()
                        }
                    }
                }
                _ => {
                    //TODO : compiler error
                    unreachable!()
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
#[derive(Clone, Debug, PartialEq, Default, Decode, Encode, Hash, Eq)]
pub struct Path(pub Vec<PathNode>);

impl Cached for Path {}

impl Path {
    pub fn inner(&self) -> &[PathNode] {
        &self.0
    }

    pub fn add_child(&mut self, input: PathNode) {
        self.0.push(input);
    }

    pub fn only_paramless(&self) -> bool {
        for i in &self.0 {
            match i.param {
                None => {}
                _ => {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Clone, Debug, PartialEq, Decode, Encode, Hash, Eq)]
pub struct PathNode {
    pub name: String,
    pub param: Option<FunctionNodeParams>,
}

#[derive(Clone, Debug, PartialEq, Decode, Encode, Hash, Eq)]
pub enum FunctionNodeParams {
    Tuple(Vec<Expression>),
    Angles(Vec<Path>),
}

#[derive(Debug, PartialEq, Decode, Encode, Hash, Clone)]
pub enum HIRNode {
    MainFunction {
        block: Block,
    },
    FunctionDeclaration {
        name: String,
        params: FunctionDeclarationParameters,
        block: Block,
        out_type: Option<Path>,
    },
    ViewportDeclaration {
        name: String,
        params: FunctionDeclarationParameters,
        block: Block,
    },

    StaticVariableDeclaration, // TODO
    ComponentDeclaration {
        name: String,
        block: ComponentDeclarationBlock,
    },
    UsingStatement {
        using: Path,
    },
    ModStatement {
        name: String,
        tree: HIRTree,
    },
    EOF,
}

impl Cached for HIRNode {}

#[derive(Debug, PartialEq, Decode, Encode, Hash, Clone)]
pub struct ComponentDeclarationBlock {
    contents: Vec<ComponentDeclarationBlockStatements>,
}

impl Cached for ComponentDeclarationBlock {}

#[derive(Debug, PartialEq, Decode, Encode, Hash, Clone)]
pub enum ComponentDeclarationBlockStatements {
    FinalVariableDeclaration {
        variablename: String,
        variabletype: Option<Path>,
        declarationexpression: Option<Expression>,
    },
    ConstVariableDeclaration,
    FunctionDeclaration {
        block: Block,
    },
    RenderBlockDeclaration {
        block: RenderBlock,
    },
    PublicConstructor {
        expression: Expression,
    },
}

impl Cached for ComponentDeclarationBlockStatements {}

#[derive(Debug, PartialEq, Decode, Encode, Hash, Clone)]
pub struct RenderBlock {
    vertices_block: VerticesBlock,
    fragments_block: FragmentsBlock,
}

impl Cached for RenderBlock {}

#[derive(Debug, PartialEq, Decode, Encode, Hash, Clone)]
pub struct VerticesBlock {
    block: Block,
}

impl Cached for VerticesBlock {}

#[derive(Debug, PartialEq, Decode, Encode, Hash, Clone)]
pub struct FragmentsBlock {
    block: Block,
}

impl Cached for FragmentsBlock {}

#[derive(Debug, PartialEq, Decode, Encode, Hash, Clone)]
pub enum FunctionBlockStatements {
    VariableDeclaration {
        variablename: String,
        variabletype: Option<Path>,
        declarationexpression: Option<Expression>,
        is_mutable: bool,
    },
    ExpressionStatement {
        expression: Expression,
    },
    ExportDeclaration {
        exporting: Exported,
        exporttype: Option<Path>,
        expression: Expression,
    },
}

impl Cached for FunctionBlockStatements {}

#[derive(Debug, PartialEq, Decode, Encode, Hash, Clone)]
pub enum Exported {
    ColorBuiltin,
    PositionBuiltin,
    Custom(String),
}

impl Cached for Exported {}

#[derive(Debug, PartialEq, Clone, Decode, Encode, Hash, Eq)]
pub enum Expression {
    NumberExpression {
        value: String,
        digittype: DigitType,
    },
    PathExpression {
        value: Path,
    },
    TupleExpression {
        values: Vec<Self>,
    },
    ListExpression {
        values: Vec<Self>,
    },
    UnaryOpExpression {
        optype: UnaryOpType,
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    TildeExpression {
        value: Box<Self>,
    },
    RequiredExpression {
        value: Box<Self>,
    },
    StringExpression {
        value: String,
    },
}

impl Cached for Expression {}

#[derive(Debug, PartialEq, Clone, Decode, Encode, Hash, Eq)]
pub enum UnaryOpType {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
}

impl Cached for UnaryOpType {}

#[derive(Debug, Decode, Encode, Hash)]
pub enum VariableType {
    Const,
    Final,
}

impl Cached for VariableType {}

#[derive(Debug, PartialEq, Decode, Encode, Hash, Clone)]
pub struct Block(Vec<FunctionBlockStatements>);

impl Cached for Block {}
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

#[derive(Debug, Hash, Decode, Encode, PartialEq, Clone)]
pub struct FunctionDeclarationParameters(Vec<FunctionDeclarationParameter>);
impl Cached for FunctionDeclarationParameters {}

impl FunctionDeclarationParameters {
    pub(crate) fn new() -> Self {
        Self(Vec::with_capacity(10))
    }
    pub(crate) fn push(&mut self, val: FunctionDeclarationParameter) {
        self.0.push(val);
    }
    pub fn handle(&self) -> &[FunctionDeclarationParameter] {
        &self.0
    }
}

#[derive(Debug, Hash, Decode, Encode, PartialEq, Clone)]
pub struct FunctionDeclarationParameter {
    name: String,
    pub arg_type: Path,
}
impl Cached for FunctionDeclarationParameter {}
