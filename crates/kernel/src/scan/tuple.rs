use super::Scan;
use crate::Node;
use grammar::Kind;

impl Scan<'_> {
    pub(super) fn tuple(&mut self, node: &Node) {
        if node.kind != Kind::Tuple || self.config.exempt(&self.source.path, "tuple") {
            return;
        }
        let count = node
            .kids
            .iter()
            .filter(|kid| kid.kind == Kind::Position)
            .count();
        if count <= self.config.file.limit.tuple {
            return;
        }
        let note = format!(
            "{count} positions over limit {}, the positional product needs a name; see: ectropy cookbook tuple",
            self.config.file.limit.tuple
        );
        self.mark(node.span.start, "tuple", &note);
    }
}
