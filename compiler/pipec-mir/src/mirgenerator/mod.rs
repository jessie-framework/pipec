// use pipec_ast::hir::{HIRNode, Path, hirtree::HIRTree};

// pub struct MIRGenerator<'this, T: Module> {
//     stream: &'this mut HIRTree,
//     scope: FileScope,
//     variant: std::marker::PhantomData<T>,
// }

// pub struct SubModule;
// pub struct MainModule;
// pub trait Module {}
// impl Module for SubModule {}
// impl Module for MainModule {}

// impl<'this, T: Module> MIRGenerator<'this, T> {
//     #[inline]
//     pub(crate) fn advance_stream(&mut self) -> Option<&HIRNode> {
//         self.stream.next_node()
//     }

//     #[inline]
//     pub(crate) fn peek_stream(&mut self) -> Option<&HIRNode> {
//         self.stream.peek()
//     }

//     pub fn new(stream: &'this mut HIRTree) -> Self {
//         Self {
//             stream,
//             scope: FileScope::default(),
//             variant: std::marker::PhantomData,
//         }
//     }

//     pub fn consume_next(&mut self) -> MIRNode {
//         loop {
//             let next = self.peek_stream();
//             match next {
//                 Some(v) => match v {
//                     HIRNode::UsingStatement { .. } => {
//                         self.consume_use_statement();
//                         continue;
//                     }
//                     HIRNode::ModStatement { .. } => {
//                         self.consume_mod_statement();
//                     }
//                     _ => {
//                         todo!()
//                     }
//                 },
//                 None => return MIRNode::EOF,
//             }
//         }
//     }
//     #[inline]
//     pub(crate) fn consume_mod_statement(&mut self) {

//     }

//     #[inline]
//     pub(crate) fn consume_use_statement(&mut self) {
//         if let Some(HIRNode::UsingStatement { using }) = self.advance_stream() {
//             let using = using.to_owned(); // its not cloning its making it owned
//             if !using.only_paramless() {
//                 //TODO : compile error
//                 unreachable!()
//             }
//             self.add_to_scope(using);
//         }
//         //TODO : compiler error
//         unreachable!();
//     }

//     #[inline]
//     pub(crate) fn add_to_scope(&mut self, input: Path) {
//         self.scope.push(input);
//     }
// }

// #[repr(transparent)]
// #[derive(Hash, Clone, Copy)]
// pub struct TypeId(u16);

// #[derive(Default)]
// /// Represents a file wide scope , used to manage using statements.
// pub struct FileScope {
//     using: Vec<Path>,
// }

// impl FileScope {
//     #[inline]
//     pub fn push(&mut self, input: Path) {
//         self.using.push(input);
//     }
// }

// pub enum MIRNode {
//     EOF,
// }
