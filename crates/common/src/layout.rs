use strum::IntoStaticStr;

#[derive(Debug, PartialEq, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Layout {
    RecursiveWithPoseidon,
    Starknet,
}
