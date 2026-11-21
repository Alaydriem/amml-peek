/// What the preview made of a box when it loaded it: the items placed and the bundles among them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoxSummary {
    pub items: usize,
    pub bundles: usize,
}
