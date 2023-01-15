use crate::types;

use super::Connector;
use std::io::Result;

pub(crate) struct Helm;

impl Connector for Helm {
    fn get_app(&self) -> Result<Vec<types::HelmChart>> {
        todo!()
    }
    fn sync_repos(&self) -> Result<()> {
        todo!()
    }

    type ConnectorType = Helm;
}

impl Helm {
    pub(crate) fn init() -> Helm {
        Helm
    }
}
