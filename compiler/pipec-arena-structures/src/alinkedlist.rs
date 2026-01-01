use pipec_arena::ASpan;

pub struct ALinkedList<T> {
    pub current: T,
    pub next: Option<ASpan<T>>,
}
