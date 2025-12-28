pub mod error;
pub mod hash;
pub mod object;
pub mod repository;
pub mod storage;
pub mod index;
pub mod refs;
pub mod diff;
pub mod ignore;
pub mod module;
pub mod hooks;
pub mod protocol;
pub mod remote;

pub use error::{Error, Result};
pub use hash::{Hash, Oid};
pub use object::{Blob, Tree, TreeEntry, Commit, Object, ObjectType};
pub use repository::Repository;
pub use diff::{Change, FileDiff, Hunk, DiffEngine};
pub use module::{ModuleManifest, ModuleMetadata, ModuleManager, ModuleType};
pub use hooks::{HookManager, HookType, HookResult};
pub use protocol::{Request, Response, ObjectData, RefUpdate, ObjectType as ProtocolObjectType};
pub use remote::{RemoteConfig, RemoteManager, RemoteClient};

