pub trait Entity {
    type SysId: SysId;
}

pub trait SysId: Eq + Clone + std::fmt::Debug + std::hash::Hash {
    fn generate() -> Self;
}
