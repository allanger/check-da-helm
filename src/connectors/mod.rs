mod argo;
mod helm;
mod helmfile;

use std::io::Result;
use crate::types;

pub (crate) use self::argo::Argo;
pub (crate) use self::helm::Helm;
pub (crate) use self::helmfile::Helmfile;

pub(crate) trait Connector {
    type ConnectorType;
    fn get_app(&self) -> Result<Vec<types::HelmChart>>;
    fn sync_repos(&self) -> Result<()>;
}
