// GVC Server - HTTP API for remote repository access

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use gvc_core::protocol::{Request, Response as GvcResponse, ObjectData, ObjectType};
use gvc_core::{Repository, Object};
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Server state
    let state = Arc::new(ServerState::new("./repositories"));

    // Build router
    let app = Router::new()
        .route("/api/v1/gvc", post(handle_request))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Bind and serve
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("GVC Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Server state
struct ServerState {
    root_path: PathBuf,
    repositories: RwLock<HashMap<String, Arc<Repository>>>,
}

impl ServerState {
    fn new(root_path: impl AsRef<Path>) -> Self {
        let root_path = root_path.as_ref().to_path_buf();
        std::fs::create_dir_all(&root_path).ok();
        
        Self {
            root_path,
            repositories: RwLock::new(HashMap::new()),
        }
    }

    /// Get or load a repository
    fn get_repository(&self, name: &str) -> Result<Arc<Repository>, ServerError> {
        // Check cache first
        {
            let repos = self.repositories.read().unwrap();
            if let Some(repo) = repos.get(name) {
                return Ok(Arc::clone(repo));
            }
        }

        // Load repository
        let repo_path = self.root_path.join(name);
        if !repo_path.exists() {
            return Err(ServerError::RepositoryNotFound(name.to_string()));
        }

        let repo = Repository::open(&repo_path)
            .map_err(|e| ServerError::Internal(e.to_string()))?;
        
        let repo = Arc::new(repo);

        // Cache it
        {
            let mut repos = self.repositories.write().unwrap();
            repos.insert(name.to_string(), Arc::clone(&repo));
        }

        Ok(repo)
    }
}

/// Handle GVC protocol request
async fn handle_request(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<Request>,
) -> Result<Json<GvcResponse>, ServerError> {
    info!("Handling request: {:?}", request);

    let response = match request {
        Request::ListRefs { repository } => {
            handle_list_refs(&state, &repository)?
        }
        Request::GetObjects { repository, oids } => {
            handle_get_objects(&state, &repository, &oids)?
        }
        Request::Push { repository, objects, ref_updates } => {
            handle_push(&state, &repository, objects, ref_updates)?
        }
        Request::InfoRefs { repository } => {
            handle_info_refs(&state, &repository)?
        }
    };

    Ok(Json(response))
}

/// List all references in a repository
fn handle_list_refs(state: &ServerState, repo_name: &str) -> Result<GvcResponse, ServerError> {
    let repo = state.get_repository(repo_name)?;
    
    let mut refs = HashMap::new();
    
    // Get all branches
    for branch_name in repo.list_branches()
        .map_err(|e| ServerError::Internal(e.to_string()))? 
    {
        if let Some(oid) = repo.resolve_ref(&format!("refs/heads/{}", branch_name))
            .map_err(|e| ServerError::Internal(e.to_string()))? 
        {
            refs.insert(format!("refs/heads/{}", branch_name), oid);
        }
    }
    
    // Get all tags
    for tag_name in repo.list_tags()
        .map_err(|e| ServerError::Internal(e.to_string()))?
    {
        if let Some(oid) = repo.resolve_ref(&format!("refs/tags/{}", tag_name))
            .map_err(|e| ServerError::Internal(e.to_string()))?
        {
            refs.insert(format!("refs/tags/{}", tag_name), oid);
        }
    }
    
    // Get HEAD
    let head = repo.get_head()
        .map_err(|e| ServerError::Internal(e.to_string()))?;
    
    Ok(GvcResponse::Refs { refs, head })
}

/// Get objects by their OIDs
fn handle_get_objects(
    state: &ServerState,
    repo_name: &str,
    oids: &[gvc_core::hash::Oid],
) -> Result<GvcResponse, ServerError> {
    let repo = state.get_repository(repo_name)?;
    
    let mut objects = Vec::new();
    
    for oid in oids {
        let obj = repo.read_object(oid)
            .map_err(|e| ServerError::ObjectNotFound(oid.to_string(), e.to_string()))?;
        
        let data = repo.read_object_raw(oid)
            .map_err(|e| ServerError::Internal(e.to_string()))?;
        
        let object_type = match obj {
            Object::Blob(_) => ObjectType::Blob,
            Object::Tree(_) => ObjectType::Tree,
            Object::Commit(_) => ObjectType::Commit,
        };
        
        objects.push(ObjectData {
            oid: oid.clone(),
            data,
            object_type,
        });
    }
    
    Ok(GvcResponse::Objects { objects })
}

/// Handle push operation
fn handle_push(
    state: &ServerState,
    repo_name: &str,
    objects: Vec<ObjectData>,
    ref_updates: Vec<gvc_core::protocol::RefUpdate>,
) -> Result<GvcResponse, ServerError> {
    let repo = state.get_repository(repo_name)?;
    
    // Write all objects
    for obj_data in objects {
        repo.write_object_raw(&obj_data.oid, &obj_data.data)
            .map_err(|e| ServerError::Internal(e.to_string()))?;
    }
    
    // Update references
    let mut updated_refs = Vec::new();
    
    for ref_update in ref_updates {
        // Safety check: verify old_oid if provided
        if let Some(old_oid) = &ref_update.old_oid {
            let current = repo.resolve_ref(&ref_update.name)
                .map_err(|e| ServerError::Internal(e.to_string()))?;
            
            if current.as_ref() != Some(old_oid) && !ref_update.force {
                return Ok(GvcResponse::PushResult {
                    success: false,
                    updated_refs: vec![],
                    message: format!(
                        "Reference {} has changed (use --force to override)",
                        ref_update.name
                    ),
                });
            }
        }
        
        // Update reference
        repo.update_ref(&ref_update.name, &ref_update.new_oid)
            .map_err(|e| ServerError::Internal(e.to_string()))?;
        
        updated_refs.push(ref_update.name.clone());
    }
    
    Ok(GvcResponse::PushResult {
        success: true,
        updated_refs,
        message: "Push successful".to_string(),
    })
}

/// Get repository info
fn handle_info_refs(state: &ServerState, repo_name: &str) -> Result<GvcResponse, ServerError> {
    let repo_path = state.root_path.join(repo_name);
    
    if !repo_path.exists() {
        return Ok(GvcResponse::RepoInfo {
            exists: false,
            head: None,
            branches: vec![],
        });
    }
    
    let repo = state.get_repository(repo_name)?;
    
    let head = repo.get_head()
        .map_err(|e| ServerError::Internal(e.to_string()))?;
    
    let branches = repo.list_branches()
        .map_err(|e| ServerError::Internal(e.to_string()))?;
    
    Ok(GvcResponse::RepoInfo {
        exists: true,
        head,
        branches,
    })
}

/// Server errors
#[derive(Debug)]
enum ServerError {
    RepositoryNotFound(String),
    ObjectNotFound(String, String),
    Internal(String),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ServerError::RepositoryNotFound(name) => (
                StatusCode::NOT_FOUND,
                "REPO_NOT_FOUND",
                format!("Repository not found: {}", name),
            ),
            ServerError::ObjectNotFound(oid, err) => (
                StatusCode::NOT_FOUND,
                "OBJECT_NOT_FOUND",
                format!("Object {} not found: {}", oid, err),
            ),
            ServerError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                msg,
            ),
        };

        let response = GvcResponse::Error {
            code: code.to_string(),
            message,
        };

        (status, Json(response)).into_response()
    }
}
