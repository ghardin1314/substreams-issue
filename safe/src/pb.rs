#[rustfmt::skip]
#[path = "../target/pb/masterfile.common.v1.rs"]
pub(in crate::pb) mod common_v1;

pub mod common {
    pub mod v1 {
        pub use super::super::common_v1::*;
    }
}

#[rustfmt::skip]
#[path = "../target/pb/masterfile.deployment.v1.rs"]
pub(in crate::pb) mod deployment_v1;

pub mod deployment {
    pub mod v1 {
        pub use super::super::deployment_v1::*;
    }
}

#[rustfmt::skip]
#[path = "../target/pb/masterfile.factory.v1.rs"]
pub(in crate::pb) mod factory_v1;

pub mod factory {
    pub mod v1 {
        pub use super::super::factory_v1::*;
    }
}

#[rustfmt::skip]
#[path = "../target/pb/masterfile.safe.v1.rs"]
pub(in crate::pb) mod safe_v1;

pub mod safe {
    pub mod v1 {
        pub use super::super::safe_v1::*;
    }
}
