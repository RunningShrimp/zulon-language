use crate::air::AirModule;

pub trait Backend {
    type Artifact;
    fn codegen(&mut self, module: &AirModule) -> Result<Self::Artifact, String>;
}

pub struct BackendOptions {
    pub opt_level: u8,
}

impl Default for BackendOptions {
    fn default() -> Self {
        Self { opt_level: 0 }
    }
}
