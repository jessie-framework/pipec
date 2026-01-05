#![allow(unused_must_use)]
use pipec_arena::{ASpan, Arena};
use pipec_arena_structures::{ADynList, AVec};
use pipec_file_loader::{FileId, FileLoader};
use pipec_span::Span;
use std::path::PathBuf;

use crate::RecursiveGuard;
use crate::ast::asttree::ASTTree;
use crate::tokenizer::DigitType;
use crate::tokenizer::Token;
use crate::tokenizer::Tokenizer;
use crate::tokenizer::tokentree::TokenTree;
pub mod asttree;

pub struct ASTGenerator<'this> {
    src: FileId,
    tokens: &'this mut TokenTree<'this>,
    guard: &'this mut RecursiveGuard,
    pub arena: &'this mut Arena,
    pub loader: &'this mut FileLoader,
    path: PathBuf,
}

impl<'this> ASTGenerator<'this> {
    pub fn tree(mut self) -> ASTTree {
        let out = self.arena.alloc(AVec::new());
        let out_handle = self.arena.take(out);
        loop {
            let next = self.parse_value();
            if matches!(next, ASTNode::EOF) {
                break;
            }
            out_handle.push(next).unwrap();
        }
        ASTTree::new(out, self.src)
    }

    pub fn file_id(&self) -> FileId {
        self.src
    }
    pub fn new(
        src: FileId,
        tokens: &'this mut TokenTree<'this>,
        arena: &'this mut Arena,
        path: PathBuf,
        guard: &'this mut RecursiveGuard,
        loader: &'this mut FileLoader,
    ) -> Self {
        let path = path.parent().unwrap().to_path_buf();
        Self {
            src,
            arena,
            tokens,
            path,
            guard,
            loader,
        }
    }

    #[inline]
    pub(crate) fn advance_stream(&mut self) -> Option<Token> {
        println!("{:#?}", self.peek_stream());
        self.tokens.next_token()
    }
    #[inline]
    pub(crate) fn peek_stream(&mut self) -> &Option<Token> {
        self.tokens.peek()
    }

    pub fn parse_value(&mut self) -> ASTNode {
        self.consume_whitespace();
        match self.peek_stream() {
            Some(v) => match v {
                Token::UsingKeyword => self.consume_using_keyword(),
                Token::ModuleKeyword => self.consume_module_keyword(),
                Token::ComponentKeyword => self.consume_component_keyword(),
                Token::ViewportKeyword => self.consume_viewport_keyword(),
                Token::FunctionKeyword => self.consume_function_keyword(),
                Token::PublicKeyword => self.consume_public_keyword(),
                _v => {
                    println!("{_v:#?}");
                    todo!();
                }
            },
            None => ASTNode::EOF,
        }
    }

    #[inline]
    pub(crate) fn consume_public_keyword(&mut self) -> ASTNode {
        self.advance_stream();
        let val = self.parse_value();
        ASTNode::Public(self.arena.alloc(val))
    }

    #[inline]
    pub(crate) fn consume_function_keyword(&mut self) -> ASTNode {
        self.advance_stream();
        self.consume_whitespace();
        let name = self.must_ident();
        self.consume_whitespace();
        let params = self.consume_function_parameters();
        self.consume_whitespace();
        let mut out_type = None;
        if self.next_is(Token::FatArrow) {
            self.advance_stream();
            self.consume_whitespace();
            out_type = Some(self.consume_a_path());
        }
        self.consume_whitespace();
        let block = self.consume_function_block();

        ASTNode::FunctionDeclaration {
            name,
            params,
            block,
            out_type,
        }
    }

    #[inline]
    pub(crate) fn consume_viewport_keyword(&mut self) -> ASTNode {
        self.advance_stream();
        self.consume_whitespace();
        let name = self.must_ident();
        self.consume_whitespace();
        let params = self.consume_function_parameters();
        self.consume_whitespace();
        let block = self.consume_function_block();
        ASTNode::ViewportDeclaration {
            name,
            params,
            block,
        }
    }

    #[inline]
    pub(crate) fn next_is(&mut self, next: Token) -> bool {
        self.peek_stream() == &Some(next)
    }

    #[inline]
    pub(crate) fn consume_function_parameters(&mut self) -> FunctionDeclarationParameters {
        let vec = self.arena.alloc(AVec::new());
        let vec_handle = self.arena.take(vec);
        self.must(Token::LeftParenthesis);
        loop {
            self.consume_whitespace();
            if self.next_is(Token::RightParenthesis) {
                self.advance_stream();
                break;
            }
            vec_handle.push(self.consume_function_parameter());
            self.consume_whitespace();
            if self.next_is(Token::Comma) {
                self.advance_stream();
                continue;
            }
        }
        FunctionDeclarationParameters(vec)
    }

    #[inline]
    pub(crate) fn must_ident(&mut self) -> Span {
        if let Some(Token::Ident(v)) = self.advance_stream() {
            return v;
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
        println!("should have {:#?}", self.peek_stream());
        if self.advance_stream() != Some(val) {
            // TODO : compiler error
            unreachable!()
        }
    }

    #[inline]
    pub(crate) fn consume_module_keyword(&mut self) -> ASTNode {
        self.advance_stream();
        self.consume_whitespace();
        let mod_path = self.must_ident();
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
    pub(crate) fn consume_mod_block(&mut self, mod_path: Span) -> ASTNode {
        self.advance_stream();
        let nodes = self.arena.alloc(AVec::new());
        let nodes_handle = self.arena.take(nodes);
        loop {
            self.consume_whitespace();
            if self.peek_stream() == &Some(Token::RightCurly) {
                self.advance_stream();
                break;
            }
            nodes_handle.push(self.parse_value());
        }
        ASTNode::ModStatement {
            name: mod_path,
            tree: ASTTree::new(nodes, self.src),
        }
    }

    #[inline]
    pub(crate) fn consume_node_from_fs(&mut self, mod_path: Span) -> ASTNode {
        let src = self.loader.load(self.src);
        self.advance_stream();
        let path1 = {
            let mut cloned = self.path.clone();
            cloned.push(format!(
                "{}/mod.pipec",
                mod_path.parse_arena(src, self.arena)
            ));
            cloned
        };
        let path2 = {
            let mut cloned = self.path.clone();
            cloned.push(format!("{}.pipec", mod_path.parse_arena(src, self.arena)));
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
            let file_id = self.loader.open(&path1, self.arena).unwrap();
            let file_contents = self.loader.load(file_id);
            let src = self.arena.take_str_slice(file_contents);

            let mut tokentree = Tokenizer::new(src).tree();

            let ast_generator = ASTGenerator::new(
                file_id,
                &mut tokentree,
                self.arena,
                path1,
                self.guard,
                self.loader,
            );
            let tree = ast_generator.tree();
            return ASTNode::ModStatement {
                name: mod_path,
                tree,
            };
        }

        if path2.exists() {
            let file_id = self.loader.open(&path2, self.arena).unwrap();
            let file_contents = self.loader.load(file_id);
            let src = self.arena.take_str_slice(file_contents);

            let mut tokentree = Tokenizer::new(src).tree();

            let ast_generator = ASTGenerator::new(
                file_id,
                &mut tokentree,
                self.arena,
                path1,
                self.guard,
                self.loader,
            );
            let tree = ast_generator.tree();
            return ASTNode::ModStatement {
                name: mod_path,
                tree,
            };
        }

        println!("{path1:#?},{path2:#?}");
        unreachable!()
    }

    #[inline]
    pub(crate) fn consume_component_keyword(&mut self) -> ASTNode {
        self.advance_stream();
        self.consume_whitespace();
        if let Some(Token::Ident(v)) = self.advance_stream() {
            return ASTNode::ComponentDeclaration {
                name: v,
                block: self.consume_component_declaration_block(),
            };
        }
        //TODO : compiler error
        unreachable!();
    }

    #[inline]
    pub(crate) fn consume_component_declaration_block(&mut self) -> ComponentDeclarationBlock {
        self.consume_whitespace();
        self.must(Token::LeftCurly);
        let mut contents = ADynList::new(self.arena);
        loop {
            self.consume_whitespace();
            let next = self.peek_stream();
            if next == &Some(Token::RightCurly) {
                self.advance_stream();
                break;
            }
            contents.push(self.consume_component_declaration_statement(), self.arena);
        }

        ComponentDeclarationBlock { contents }
    }

    #[inline]
    pub(crate) fn consume_component_declaration_statement(
        &mut self,
    ) -> ComponentDeclarationBlockStatements {
        match self.advance_stream() {
            Some(Token::FinalKeyword) => self.consume_final_variable_declaration(),
            Some(Token::RenderKeyword) => self.consume_component_render_block(),
            _v => {
                //TODO : compiler error
                unreachable!();
            }
        }
    }

    #[inline]
    pub(crate) fn consume_component_render_block(&mut self) -> ComponentDeclarationBlockStatements {
        self.consume_whitespace();
        let block = self.consume_component_render_block_inner();
        ComponentDeclarationBlockStatements::RenderBlockDeclaration { block }
    }

    #[inline]
    pub(crate) fn consume_component_render_block_inner(&mut self) -> RenderBlock {
        self.must(Token::LeftCurly);
        self.consume_whitespace();
        let vertices_block = self.consume_vertices_block();
        self.consume_whitespace();
        let fragments_block = self.consume_fragments_block();
        self.consume_whitespace();
        self.must(Token::RightCurly);
        RenderBlock {
            vertices_block,
            fragments_block,
        }
    }

    #[inline]
    pub(crate) fn consume_vertices_block(&mut self) -> VerticesBlock {
        self.must(Token::VerticesKeyword);
        self.consume_whitespace();
        VerticesBlock {
            block: self.consume_function_block(),
        }
    }

    #[inline]
    pub(crate) fn consume_fragments_block(&mut self) -> FragmentsBlock {
        self.must(Token::FragmentsKeyword);
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
        let variablename = self.must_ident();
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
        self.must(Token::LeftCurly);
        let mut block = ADynList::new(self.arena);
        loop {
            self.consume_whitespace();
            let next = self.peek_stream();
            if next == &Some(Token::RightCurly) {
                self.advance_stream();
                break;
            }
            block.push(self.consume_a_block_statement(), self.arena);
        }
        Block(block)
    }

    #[inline]
    pub(crate) fn consume_a_block_statement(&mut self) -> FunctionBlockStatements {
        self.consume_whitespace();
        match self.peek_stream() {
            Some(v) => match v {
                Token::MutableKeyword => self.consume_mutable_variable_declaration(),
                Token::ImmutableKeyword => self.consume_immutable_variable_declaration(),
                Token::ExportKeyword => self.consume_export_declaration(),
                Token::RenderKeyword => self.consume_render_block(),
                _ => self.consume_expression_statement(),
            },
            None => {
                // TODO : compiler error
                unreachable!()
            }
        }
    }
    #[inline]
    pub(crate) fn consume_render_block(&mut self) -> FunctionBlockStatements {
        self.advance_stream();
        self.consume_whitespace();
        FunctionBlockStatements::RenderBlock {
            block: self.consume_function_block(),
        }
    }

    #[inline]
    pub(crate) fn consume_expression_statement(&mut self) -> FunctionBlockStatements {
        let expression = self.consume_an_expression();
        self.consume_whitespace();
        let mut hidden = false;
        if self.next_is(Token::Semicolon) {
            hidden = true;
            self.advance_stream();
        }
        FunctionBlockStatements::ExpressionStatement { expression, hidden }
    }

    #[inline]
    pub(crate) fn consume_export_declaration(&mut self) -> FunctionBlockStatements {
        let src = self.loader.load(self.src);
        self.advance_stream();
        self.consume_whitespace();
        let exporting: Exported = match self.advance_stream() {
            Some(Token::Hash) => match self.advance_stream() {
                Some(Token::Ident(name)) => match name.parse_arena(src, self.arena) {
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
            Some(Token::Ident(name)) => Exported::Custom(name),
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
                if self.advance_stream() == Some(Token::EqualSign) {
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
    pub(crate) fn consume_mutable_variable_declaration(&mut self) -> FunctionBlockStatements {
        self.advance_stream();
        // mutable x : u32 = 0;
        self.consume_whitespace();
        let varname = self.must_ident();
        let vartype: Option<Path>;
        let declexpr: Option<Expression>;
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

        FunctionBlockStatements::MutableVariableDeclaration {
            variablename: varname,
            variabletype: vartype,
            declarationexpression: declexpr,
        }
        // TODO : update this function
    }
    #[inline]
    pub(crate) fn consume_immutable_variable_declaration(&mut self) -> FunctionBlockStatements {
        self.advance_stream();
        // mutable x : u32 = 0;
        self.consume_whitespace();
        let varname = self.must_ident();
        let vartype: Option<Path>;
        let declexpr: Option<Expression>;
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

        FunctionBlockStatements::ImmutableVariableDeclaration {
            variablename: varname,
            variabletype: vartype,
            declarationexpression: declexpr,
        }
        // TODO : update this function
    }

    #[inline]
    pub(crate) fn consume_a_semicolon(&mut self) {
        let peek = self.peek_stream();
        if peek == &Some(Token::Semicolon) {
            self.advance_stream();
            return;
        }
        //TODO : compiler error
        todo!();
    }

    #[inline]
    pub(crate) fn consume_an_expression(&mut self) -> Expression {
        self.consume_whitespace();
        let out = match self.peek_stream() {
            Some(Token::Digit { .. }) => self.consume_number_expression(),
            Some(Token::String(_)) => self.consume_string_expression(),
            Some(Token::LeftParenthesis) => self.consume_tuple_expression(),
            Some(Token::LeftSquare) => self.consume_list_expression(),
            Some(Token::Tilde) => self.consume_tilde_expression(),
            Some(Token::Ident(_)) => self.consume_path_expression(),
            Some(Token::RequiredKeyword) => self.consume_required_expression(),
            Some(Token::SwitchKeyword) => self.consume_switch_expression(),

            _v => {
                println!("{_v:#?}");
                //TODO : compiler error
                unreachable!();
            }
        };
        self.check_expression_rhs(out)
    }

    #[inline]
    pub(crate) fn check_expression_rhs(&mut self, input: Expression) -> Expression {
        self.consume_whitespace();
        let exprtype = match self.peek_stream() {
            Some(Token::Plus) => Some(BinaryOpType::Add),
            Some(Token::Minus) => Some(BinaryOpType::Subtract),
            Some(Token::Asterisk) => Some(BinaryOpType::Multiply),
            Some(Token::Slash) => Some(BinaryOpType::Divide),
            Some(Token::PlusEqual) => Some(BinaryOpType::AddEqual),
            Some(Token::MinusEqual) => Some(BinaryOpType::SubtractEqual),
            Some(Token::AsteriskEqual) => Some(BinaryOpType::MultiplyEqual),
            Some(Token::SlashEqual) => Some(BinaryOpType::DivideEqual),
            Some(Token::ModEqual) => Some(BinaryOpType::ModEqual),
            _ => None,
        };
        if let Some(v) = exprtype {
            self.advance_stream();
            let rhs_expr = self.consume_an_expression();

            return Expression::BinaryOpExpression {
                optype: v,
                lhs: self.arena.alloc(input),
                rhs: self.arena.alloc(rhs_expr),
            };
        }
        input
    }

    #[inline]
    pub(crate) fn consume_switch_expression(&mut self) -> Expression {
        self.advance_stream();
        self.consume_whitespace();
        let expression = self.consume_an_expression();
        let predicate = self.arena.alloc(expression);
        Expression::SwitchExpression {
            predicate,
            block: self.consume_switch_block(),
        }
    }

    #[inline]
    pub(crate) fn consume_switch_block(&mut self) -> SwitchExpressionBlock {
        self.consume_whitespace();
        self.must(Token::LeftCurly);
        let mut out = ADynList::new(self.arena);
        loop {
            self.consume_whitespace();
            if self.next_is(Token::RightCurly) {
                self.advance_stream();
                break;
            }
            out.push(self.consume_switch_arm(), self.arena);
            if self.next_is(Token::Comma) {
                self.advance_stream();
                continue;
            }
        }
        SwitchExpressionBlock(out)
    }

    #[inline]
    pub(crate) fn consume_switch_arm(&mut self) -> SwitchArm {
        let expr = self.consume_an_expression();
        println!("arm lhs = {expr:#?}");
        let lhs = self.arena.alloc(expr);
        self.consume_whitespace();
        self.must(Token::ThinArrow);
        self.consume_whitespace();
        let expr = self.consume_an_expression();
        let rhs = self.arena.alloc(expr);
        SwitchArm { lhs, rhs }
    }

    #[inline]
    pub(crate) fn consume_required_expression(&mut self) -> Expression {
        self.advance_stream();
        self.consume_whitespace();
        let expr = self.consume_an_expression();
        let value = self.arena.alloc(expr);
        Expression::RequiredExpression { value }
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

        first
    }
    #[inline]
    pub(crate) fn consume_list_expression(&mut self) -> Expression {
        self.advance_stream();
        let mut exprs = ADynList::new(self.arena);
        loop {
            exprs.push(self.consume_an_expression(), self.arena);

            self.consume_whitespace();

            let next = self.peek_stream();
            match next {
                Some(Token::Comma) => {
                    self.advance_stream();
                    continue;
                }
                Some(Token::RightSquare) => {
                    self.advance_stream();
                    return Expression::ListExpression { values: exprs };
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
        let expr = self.consume_an_expression();
        Expression::TildeExpression {
            value: self.arena.alloc(expr),
        }
    }

    #[inline]
    pub(crate) fn consume_string_expression(&mut self) -> Expression {
        Expression::PathExpression {
            value: self.consume_a_path(),
        }
    }

    #[inline]
    pub(crate) fn consume_tuple_expression(&mut self) -> Expression {
        self.advance_stream();
        let mut values = ADynList::new(self.arena);
        loop {
            values.push(self.consume_an_expression(), self.arena);
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
        match self.advance_stream() {
            Some(Token::Digit {
                val: value,
                digittype,
            }) => Expression::NumberExpression { value, digittype },
            _v => {
                //TODO : compile error
                unreachable!()
            }
        }
    }

    #[inline]
    pub(crate) fn consume_using_keyword(&mut self) -> ASTNode {
        self.advance_stream();
        self.consume_whitespace();

        let using = self.consume_a_path();
        self.consume_a_semicolon();
        ASTNode::UsingStatement { using }
    }

    #[inline]
    fn consume_a_path(&mut self) -> Path {
        let mut out = ADynList::new(self.arena);
        loop {
            let next = self.peek_stream();
            match next {
                Some(Token::Ident(v)) => {
                    let name = *v;
                    self.advance_stream();
                    let param = self.consume_path_param();
                    out.push(PathNode::Named { name, param }, self.arena);
                    continue;
                }
                Some(Token::Slash) => {
                    self.advance_stream();
                    continue;
                }
                Some(Token::LeftParenthesis) => {
                    self.advance_stream();
                    let mut vals = ADynList::new(self.arena);
                    loop {
                        self.consume_whitespace();
                        match self.advance_stream() {
                            Some(Token::Ident(v)) => {
                                vals.push(v, self.arena);
                                continue;
                            }
                            Some(Token::RightParenthesis) => {
                                break;
                            }
                            Some(Token::Comma) => {
                                continue;
                            }
                            _v => {}
                        }
                    }
                }
                _ => {
                    break;
                }
            }
        }
        Path(out)
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
    pub(crate) fn consume_whitespace(&mut self) {
        while self.tokens.peek() == &Some(Token::Whitespace) {
            self.tokens.next_token();
        }
    }

    #[inline]
    pub(crate) fn consume_angle_params(&mut self) -> ADynList<Path> {
        self.must(Token::LeftAngle);
        let mut out = ADynList::new(self.arena);
        loop {
            self.consume_whitespace();
            match self.peek_stream() {
                Some(Token::Ident(_)) => {
                    out.push(self.consume_a_path(), self.arena);
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
}

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Path(pub ADynList<PathNode>);

#[derive(Debug, Clone)]
pub enum PathNode {
    Named {
        name: Span,
        param: Option<FunctionNodeParams>,
    },
    Tuple(ADynList<Span>),
}

#[derive(Debug, Clone)]
pub enum FunctionNodeParams {
    Tuple(ADynList<Expression>),
    Angles(ADynList<Path>),
}

#[derive(Debug, Clone)]
pub enum ASTNode {
    MainFunction {
        block: Block,
    },
    FunctionDeclaration {
        name: Span,
        params: FunctionDeclarationParameters,
        block: Block,
        out_type: Option<Path>,
    },
    ViewportDeclaration {
        name: Span,
        params: FunctionDeclarationParameters,
        block: Block,
    },

    StaticVariableDeclaration, // TODO
    ComponentDeclaration {
        name: Span,
        block: ComponentDeclarationBlock,
    },
    UsingStatement {
        using: Path,
    },
    ModStatement {
        name: Span,
        tree: ASTTree,
    },
    Public(ASpan<Self>),
    EOF,
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct ComponentDeclarationBlock {
    contents: ADynList<ComponentDeclarationBlockStatements>,
}

#[derive(Debug, Clone)]
pub enum ComponentDeclarationBlockStatements {
    FinalVariableDeclaration {
        variablename: Span,
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

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct RenderBlock {
    vertices_block: VerticesBlock,
    fragments_block: FragmentsBlock,
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct VerticesBlock {
    block: Block,
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct FragmentsBlock {
    block: Block,
}

#[derive(Debug, Clone)]
pub enum FunctionBlockStatements {
    MutableVariableDeclaration {
        variablename: Span,
        variabletype: Option<Path>,
        declarationexpression: Option<Expression>,
    },
    ImmutableVariableDeclaration {
        variablename: Span,
        variabletype: Option<Path>,
        declarationexpression: Option<Expression>,
    },
    ExpressionStatement {
        hidden: bool,
        expression: Expression,
    },
    ExportDeclaration {
        exporting: Exported,
        exporttype: Option<Path>,
        expression: Expression,
    },
    RenderBlock {
        block: Block,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Exported {
    ColorBuiltin,
    PositionBuiltin,
    Custom(Span),
}

#[derive(Debug, Clone)]
pub enum Expression {
    NumberExpression {
        value: Span,
        digittype: DigitType,
    },
    PathExpression {
        value: Path,
    },
    TupleExpression {
        values: ADynList<Self>,
    },
    ListExpression {
        values: ADynList<Self>,
    },
    BinaryOpExpression {
        optype: BinaryOpType,
        lhs: ASpan<Self>,
        rhs: ASpan<Self>,
    },
    TildeExpression {
        value: ASpan<Self>,
    },
    RequiredExpression {
        value: ASpan<Self>,
    },
    SwitchExpression {
        predicate: ASpan<Self>,
        block: SwitchExpressionBlock,
    },
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct SwitchExpressionBlock(ADynList<SwitchArm>);

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct SwitchArm {
    lhs: ASpan<Expression>,
    rhs: ASpan<Expression>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum BinaryOpType {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    AddEqual,
    SubtractEqual,
    MultiplyEqual,
    DivideEqual,
    ModEqual,
}

#[derive(Debug)]
pub enum VariableType {
    Const,
    Final,
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct Block(ADynList<FunctionBlockStatements>);

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct FunctionDeclarationParameters(pub ASpan<AVec<FunctionDeclarationParameter, 20>>);

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct FunctionDeclarationParameter {
    pub name: Span,
    pub arg_type: Path,
}
