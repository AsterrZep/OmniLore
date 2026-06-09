// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod db;

pub struct DbState {
    pub conn: std::sync::Mutex<Option<rusqlite::Connection>>,
    pub project_path: std::sync::Mutex<Option<String>>,
}

#[tauri::command]
fn open_project(state: tauri::State<'_, DbState>, path: String) -> Result<db::ProjectInfo, String> {
    let mut conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    let mut path_guard = state.project_path.lock().map_err(|e| e.to_string())?;
    
    // Close existing connection if any
    *conn_guard = None;
    *path_guard = None;
    
    // Open new connection
    let db_path = std::path::Path::new(&path).join("project.db");
    
    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
    
    // Initialize DB tables
    db::init_db(&conn).map_err(|e| e.to_string())?;
    
    // Check if project info exists, if not create a default one
    let project_info = db::get_project_info(&conn).map_err(|e| e.to_string())?;
    
    let info = if let Some(info) = project_info {
        info
    } else {
        // Create new project info
        let project_name = std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled Project")
            .to_string();
            
        let new_info = db::ProjectInfo {
            id: uuid::Uuid::new_v4().to_string(),
            name: project_name,
            description: Some("Created with OmniLore".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        db::save_project_info(&conn, &new_info).map_err(|e| e.to_string())?;
        new_info
    };
    
    *conn_guard = Some(conn);
    *path_guard = Some(path);
    Ok(info)
}

#[tauri::command]
fn get_project_info(state: tauri::State<'_, DbState>) -> Result<db::ProjectInfo, String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        let info = db::get_project_info(conn).map_err(|e| e.to_string())?;
        info.ok_or_else(|| "No project info available".to_string())
    } else {
        Err("No project open".to_string())
    }
}

// --- Flat-file Read/Write commands with indexation ---

#[tauri::command]
fn read_document_file(state: tauri::State<'_, DbState>, relative_path: String) -> Result<String, String> {
    let path_guard = state.project_path.lock().map_err(|e| e.to_string())?;
    if let Some(proj_path) = &*path_guard {
        let full_path = std::path::Path::new(proj_path).join(&relative_path);
        if !full_path.exists() {
            return Ok("".to_string()); // Return empty string if file doesn't exist yet
        }
        std::fs::read_to_string(full_path).map_err(|e| e.to_string())
     } else {
         Err("No project open".to_string())
     }
}

#[tauri::command]
fn write_document_file(state: tauri::State<'_, DbState>, id: String, relative_path: String, content: String) -> Result<(), String> {
    let path_guard = state.project_path.lock().map_err(|e| e.to_string())?;
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    
    if let (Some(proj_path), Some(conn)) = (&*path_guard, &*conn_guard) {
        let full_path = std::path::Path::new(proj_path).join(&relative_path);
        
        // Ensure directories exist
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        
        std::fs::write(&full_path, &content).map_err(|e| e.to_string())?;
        
        // Run indexing
        index_document_relations(conn, &id, &content)?;
        
        Ok(())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn create_document(state: tauri::State<'_, DbState>, doc: db::Document) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::create_document(conn, &doc).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn get_documents(state: tauri::State<'_, DbState>) -> Result<Vec<db::Document>, String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::get_documents(conn).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn update_document(state: tauri::State<'_, DbState>, id: String, title: String, relative_path: String) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        let updated_at = chrono::Utc::now().to_rfc3339();
        db::update_document(conn, &id, &title, &relative_path, &updated_at).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn delete_document(state: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::delete_document(conn, &id).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn create_entity(state: tauri::State<'_, DbState>, entity: db::Entity) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::create_entity(conn, &entity).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn get_entities(state: tauri::State<'_, DbState>) -> Result<Vec<db::Entity>, String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::get_entities(conn).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn update_entity(state: tauri::State<'_, DbState>, id: String, name: String, description: Option<String>, properties: serde_json::Value) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        let updated_at = chrono::Utc::now().to_rfc3339();
        db::update_entity(conn, &id, &name, description.as_deref(), &properties, &updated_at).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn delete_entity(state: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::delete_entity(conn, &id).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn create_tag(state: tauri::State<'_, DbState>, tag: db::Tag) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::create_tag(conn, &tag).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn get_tags(state: tauri::State<'_, DbState>) -> Result<Vec<db::Tag>, String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::get_tags(conn).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn delete_tag(state: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::delete_tag(conn, &id).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn associate_entity_tag(state: tauri::State<'_, DbState>, entity_id: String, tag_id: String) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::associate_entity_tag(conn, &entity_id, &tag_id).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn disassociate_entity_tag(state: tauri::State<'_, DbState>, entity_id: String, tag_id: String) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::disassociate_entity_tag(conn, &entity_id, &tag_id).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn get_entity_tags(state: tauri::State<'_, DbState>, entity_id: String) -> Result<Vec<db::Tag>, String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::get_entity_tags(conn, &entity_id).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn associate_document_tag(state: tauri::State<'_, DbState>, document_id: String, tag_id: String) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::associate_document_tag(conn, &document_id, &tag_id).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn disassociate_document_tag(state: tauri::State<'_, DbState>, document_id: String, tag_id: String) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::disassociate_document_tag(conn, &document_id, &tag_id).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn get_document_tags(state: tauri::State<'_, DbState>, document_id: String) -> Result<Vec<db::Tag>, String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::get_document_tags(conn, &document_id).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn create_link(state: tauri::State<'_, DbState>, link: db::EntityLink) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::create_link(conn, &link).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn get_links(state: tauri::State<'_, DbState>) -> Result<Vec<db::EntityLink>, String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::get_links(conn).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn delete_link(state: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::delete_link(conn, &id).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

// --- Relationship Graph Data command ---

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub connections_count: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GraphLink {
    pub id: String,
    pub source: String,
    pub target: String,
    pub link_type: String,
    pub weight: f64,
    pub description: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphLink>,
}

#[tauri::command]
fn get_graph_data(state: tauri::State<'_, DbState>) -> Result<GraphData, String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        let entities = db::get_entities(conn).map_err(|e| e.to_string())?;
        let db_links = db::get_links(conn).map_err(|e| e.to_string())?;
        
        let mut nodes = Vec::new();
        for ent in entities {
            let mut count = 0;
            for link in &db_links {
                if link.source_entity_id == ent.id || link.target_entity_id == ent.id {
                    count += 1;
                }
            }
            nodes.push(GraphNode {
                id: ent.id,
                name: ent.name,
                entity_type: ent.entity_type,
                connections_count: count,
            });
        }
        
        let graph_links = db_links.into_iter().map(|l| GraphLink {
            id: l.id,
            source: l.source_entity_id,
            target: l.target_entity_id,
            link_type: l.link_type,
            weight: l.weight,
            description: l.description,
        }).collect();
        
        Ok(GraphData {
            nodes,
            links: graph_links,
        })
    } else {
        Err("No project open".to_string())
    }
}

// --- Flow Nodes & Links commands ---

#[tauri::command]
fn create_flow_node(state: tauri::State<'_, DbState>, node: db::FlowNode) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::create_flow_node(conn, &node).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn get_flow_nodes(state: tauri::State<'_, DbState>) -> Result<Vec<db::FlowNode>, String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::get_flow_nodes(conn).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn update_flow_node_position(state: tauri::State<'_, DbState>, id: String, x: f64, y: f64) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        let updated_at = chrono::Utc::now().to_rfc3339();
        db::update_flow_node_position(conn, &id, x, y, &updated_at).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn update_flow_node(state: tauri::State<'_, DbState>, node: db::FlowNode) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        let mut n = node;
        n.updated_at = chrono::Utc::now().to_rfc3339();
        db::update_flow_node(conn, &n).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn delete_flow_node(state: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::delete_flow_node(conn, &id).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn create_flow_link(state: tauri::State<'_, DbState>, link: db::FlowLink) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::create_flow_link(conn, &link).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn get_flow_links(state: tauri::State<'_, DbState>) -> Result<Vec<db::FlowLink>, String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::get_flow_links(conn).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn delete_flow_link(state: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::delete_flow_link(conn, &id).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

// --- Timeline Events commands ---

#[tauri::command]
fn create_timeline_event(state: tauri::State<'_, DbState>, ev: db::TimelineEvent) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::create_timeline_event(conn, &ev).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn get_timeline_events(state: tauri::State<'_, DbState>) -> Result<Vec<db::TimelineEvent>, String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::get_timeline_events(conn).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn delete_timeline_event(state: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::delete_timeline_event(conn, &id).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

#[tauri::command]
fn update_timeline_event(state: tauri::State<'_, DbState>, ev: db::TimelineEvent) -> Result<(), String> {
    let conn_guard = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = &*conn_guard {
        db::update_timeline_event(conn, &ev).map_err(|e| e.to_string())
    } else {
        Err("No project open".to_string())
    }
}

// --- Autocomplete and automated text relation indexer ---

fn index_document_relations(conn: &rusqlite::Connection, doc_id: &str, content: &str) -> Result<(), String> {
    // 1. Clear old tag associations
    conn.execute("DELETE FROM document_tags WHERE document_id = ?1", rusqlite::params![doc_id])
        .map_err(|e| e.to_string())?;

    // 2. Parse words
    let mut mentioned_entities: Vec<String> = Vec::new();
    let mut mentioned_tags: Vec<String> = Vec::new();

    for word in content.split_whitespace() {
        if word.starts_with('#') && word.len() > 1 {
            let tag_name = format!("#{}", clean_token(word));
            if let Some(tag) = db::find_tag_by_name(conn, &tag_name).map_err(|e| e.to_string())? {
                if !mentioned_tags.contains(&tag.id) {
                    mentioned_tags.push(tag.id.clone());
                    db::associate_document_tag(conn, doc_id, &tag.id).map_err(|e| e.to_string())?;
                }
            }
        } else if word.starts_with('@') && word.len() > 1 {
            let ent_name = clean_token(word);
            if let Some(entity) = db::find_entity_by_name(conn, &ent_name).map_err(|e| e.to_string())? {
                if !mentioned_entities.contains(&entity.id) {
                    mentioned_entities.push(entity.id.clone());
                }
            }
        }
    }

    // 3. Associate tags with entities in this document
    for entity_id in &mentioned_entities {
        for tag_id in &mentioned_tags {
            db::associate_entity_tag(conn, entity_id, tag_id).map_err(|e| e.to_string())?;
        }
    }

    // 4. Update relationship weights for co-occurring characters
    let timestamp = chrono::Utc::now().to_rfc3339();
    for i in 0..mentioned_entities.len() {
        for j in (i + 1)..mentioned_entities.len() {
            let ent_a = &mentioned_entities[i];
            let ent_b = &mentioned_entities[j];

            if let Some(link_id) = db::link_exists(conn, ent_a, ent_b).map_err(|e| e.to_string())? {
                // Increment relationship weight
                db::increment_link_weight(conn, &link_id, 0.1, &timestamp).map_err(|e| e.to_string())?;
            } else {
                // Create a default link
                let new_link = db::EntityLink {
                    id: uuid::Uuid::new_v4().to_string(),
                    source_entity_id: ent_a.clone(),
                    target_entity_id: ent_b.clone(),
                    link_type: "custom".to_string(),
                    description: Some("Mencionados juntos en escena".to_string()),
                    weight: 1.0,
                    created_at: timestamp.clone(),
                    updated_at: timestamp.clone(),
                };
                db::create_link(conn, &new_link).map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

fn clean_token(token: &str) -> String {
    // Strip leading char (@ or #)
    let mut s = token.chars().skip(1).collect::<String>();
    // Strip trailing punctuation
    while let Some(c) = s.chars().last() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            break;
        }
        s.pop();
    }
    s
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(DbState { 
            conn: std::sync::Mutex::new(None),
            project_path: std::sync::Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            open_project,
            get_project_info,
            read_document_file,
            write_document_file,
            create_document,
            get_documents,
            update_document,
            delete_document,
            create_entity,
            get_entities,
            update_entity,
            delete_entity,
            create_tag,
            get_tags,
            delete_tag,
            associate_entity_tag,
            disassociate_entity_tag,
            get_entity_tags,
            associate_document_tag,
            disassociate_document_tag,
            get_document_tags,
            create_link,
            get_links,
            delete_link,
            get_graph_data,
            create_flow_node,
            get_flow_nodes,
            update_flow_node_position,
            update_flow_node,
            delete_flow_node,
            create_flow_link,
            get_flow_links,
            delete_flow_link,
            create_timeline_event,
            get_timeline_events,
            delete_timeline_event,
            update_timeline_event
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


