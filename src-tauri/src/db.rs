use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ==========================================
// 1. Rust Structs / Models
// ==========================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub relative_path: String,
    pub content_type: String, // "prose" | "screenplay"
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: String, // "character" | "location" | "item" | "faction" | "magic"
    pub description: Option<String>,
    pub properties: Value, // JSON block for attributes like age, loyalty, layout coords
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EntityLink {
    pub id: String,
    pub source_entity_id: String,
    pub target_entity_id: String,
    pub link_type: String, // "family" | "affiliation" | "enmity" | "love" | "custom"
    pub description: Option<String>,
    pub weight: f64,
    pub created_at: String,
    pub updated_at: String,
}

// ==========================================
// 2. Database Schema Initialization & Setup
// ==========================================

pub fn init_db(conn: &Connection) -> Result<()> {
    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON;", [])?;

    // Create tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS project_info (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            relative_path TEXT NOT NULL UNIQUE,
            content_type TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS entities (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            type TEXT NOT NULL,
            description TEXT,
            properties TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS entity_tags (
            entity_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            PRIMARY KEY (entity_id, tag_id),
            FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS document_tags (
            document_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            PRIMARY KEY (document_id, tag_id),
            FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS links (
            id TEXT PRIMARY KEY,
            source_entity_id TEXT NOT NULL,
            target_entity_id TEXT NOT NULL,
            type TEXT NOT NULL,
            description TEXT,
            weight REAL DEFAULT 1.0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (source_entity_id) REFERENCES entities(id) ON DELETE CASCADE,
            FOREIGN KEY (target_entity_id) REFERENCES entities(id) ON DELETE CASCADE
        );",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS flow_nodes (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT,
            x REAL NOT NULL,
            y REAL NOT NULL,
            conditions TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS flow_links (
            id TEXT PRIMARY KEY,
            source_node_id TEXT NOT NULL,
            target_node_id TEXT NOT NULL,
            label TEXT,
            conditions TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (source_node_id) REFERENCES flow_nodes(id) ON DELETE CASCADE,
            FOREIGN KEY (target_node_id) REFERENCES flow_nodes(id) ON DELETE CASCADE
        );",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS timeline_events (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            event_date TEXT NOT NULL,
            associated_entity_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (associated_entity_id) REFERENCES entities(id) ON DELETE SET NULL
        );",
        [],
    )?;

    Ok(())
}

// ==========================================
// 3. CRUD Implementation
// ==========================================

// --- Project Info ---
pub fn save_project_info(conn: &Connection, info: &ProjectInfo) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO project_info (id, name, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![info.id, info.name, info.description, info.created_at, info.updated_at],
    )?;
    Ok(())
}

pub fn get_project_info(conn: &Connection) -> Result<Option<ProjectInfo>> {
    let mut stmt = conn.prepare("SELECT id, name, description, created_at, updated_at FROM project_info LIMIT 1")?;
    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
        Ok(Some(ProjectInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        }))
    } else {
        Ok(None)
    }
}

// --- Documents ---
pub fn create_document(conn: &Connection, doc: &Document) -> Result<()> {
    conn.execute(
        "INSERT INTO documents (id, title, relative_path, content_type, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![doc.id, doc.title, doc.relative_path, doc.content_type, doc.created_at, doc.updated_at],
    )?;
    Ok(())
}

pub fn get_documents(conn: &Connection) -> Result<Vec<Document>> {
    let mut stmt = conn.prepare("SELECT id, title, relative_path, content_type, created_at, updated_at FROM documents")?;
    let doc_iter = stmt.query_map([], |row| {
        Ok(Document {
            id: row.get(0)?,
            title: row.get(1)?,
            relative_path: row.get(2)?,
            content_type: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;

    let mut docs = Vec::new();
    for doc in doc_iter {
        docs.push(doc?);
    }
    Ok(docs)
}

pub fn update_document(conn: &Connection, id: &str, title: &str, relative_path: &str, updated_at: &str) -> Result<()> {
    conn.execute(
        "UPDATE documents SET title = ?1, relative_path = ?2, updated_at = ?3 WHERE id = ?4",
        params![title, relative_path, updated_at, id],
    )?;
    Ok(())
}

pub fn delete_document(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM documents WHERE id = ?1", params![id])?;
    Ok(())
}

// --- Entities ---
pub fn create_entity(conn: &Connection, entity: &Entity) -> Result<()> {
    let props_str = serde_json::to_string(&entity.properties).unwrap_or_else(|_| "{}".to_string());
    conn.execute(
        "INSERT INTO entities (id, name, type, description, properties, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            entity.id,
            entity.name,
            entity.entity_type,
            entity.description,
            props_str,
            entity.created_at,
            entity.updated_at
        ],
    )?;
    Ok(())
}

pub fn get_entities(conn: &Connection) -> Result<Vec<Entity>> {
    let mut stmt = conn.prepare("SELECT id, name, type, description, properties, created_at, updated_at FROM entities")?;
    let entity_iter = stmt.query_map([], |row| {
        let props_str: String = row.get(4)?;
        let properties: Value = serde_json::from_str(&props_str).unwrap_or(Value::Null);
        Ok(Entity {
            id: row.get(0)?,
            name: row.get(1)?,
            entity_type: row.get(2)?,
            description: row.get(3)?,
            properties,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;

    let mut entities = Vec::new();
    for entity in entity_iter {
        entities.push(entity?);
    }
    Ok(entities)
}

pub fn update_entity(conn: &Connection, id: &str, name: &str, description: Option<&str>, properties: &Value, updated_at: &str) -> Result<()> {
    let props_str = serde_json::to_string(properties).unwrap_or_else(|_| "{}".to_string());
    conn.execute(
        "UPDATE entities SET name = ?1, description = ?2, properties = ?3, updated_at = ?4 WHERE id = ?5",
        params![name, description, props_str, updated_at, id],
    )?;
    Ok(())
}

pub fn delete_entity(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM entities WHERE id = ?1", params![id])?;
    Ok(())
}

// --- Tags ---
pub fn create_tag(conn: &Connection, tag: &Tag) -> Result<()> {
    conn.execute(
        "INSERT INTO tags (id, name, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![tag.id, tag.name, tag.description, tag.created_at, tag.updated_at],
    )?;
    Ok(())
}

pub fn get_tags(conn: &Connection) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare("SELECT id, name, description, created_at, updated_at FROM tags")?;
    let tag_iter = stmt.query_map([], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;

    let mut tags = Vec::new();
    for tag in tag_iter {
        tags.push(tag?);
    }
    Ok(tags)
}

pub fn delete_tag(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM tags WHERE id = ?1", params![id])?;
    Ok(())
}

// --- Entity-Tag Associations ---
pub fn associate_entity_tag(conn: &Connection, entity_id: &str, tag_id: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO entity_tags (entity_id, tag_id) VALUES (?1, ?2)",
        params![entity_id, tag_id],
    )?;
    Ok(())
}

pub fn disassociate_entity_tag(conn: &Connection, entity_id: &str, tag_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM entity_tags WHERE entity_id = ?1 AND tag_id = ?2",
        params![entity_id, tag_id],
    )?;
    Ok(())
}

pub fn get_entity_tags(conn: &Connection, entity_id: &str) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.description, t.created_at, t.updated_at
         FROM tags t
         JOIN entity_tags et ON t.id = et.tag_id
         WHERE et.entity_id = ?1"
    )?;
    let tag_iter = stmt.query_map(params![entity_id], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;

    let mut tags = Vec::new();
    for tag in tag_iter {
        tags.push(tag?);
    }
    Ok(tags)
}

// --- Document-Tag Associations ---
pub fn associate_document_tag(conn: &Connection, document_id: &str, tag_id: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO document_tags (document_id, tag_id) VALUES (?1, ?2)",
        params![document_id, tag_id],
    )?;
    Ok(())
}

pub fn disassociate_document_tag(conn: &Connection, document_id: &str, tag_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM document_tags WHERE document_id = ?1 AND tag_id = ?2",
        params![document_id, tag_id],
    )?;
    Ok(())
}

pub fn get_document_tags(conn: &Connection, document_id: &str) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.description, t.created_at, t.updated_at
         FROM tags t
         JOIN document_tags dt ON t.id = dt.tag_id
         WHERE dt.document_id = ?1"
    )?;
    let tag_iter = stmt.query_map(params![document_id], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;

    let mut tags = Vec::new();
    for tag in tag_iter {
        tags.push(tag?);
    }
    Ok(tags)
}

// --- Links (Relationships between entities) ---
pub fn create_link(conn: &Connection, link: &EntityLink) -> Result<()> {
    conn.execute(
        "INSERT INTO links (id, source_entity_id, target_entity_id, type, description, weight, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            link.id,
            link.source_entity_id,
            link.target_entity_id,
            link.link_type,
            link.description,
            link.weight,
            link.created_at,
            link.updated_at
        ],
    )?;
    Ok(())
}

pub fn get_links(conn: &Connection) -> Result<Vec<EntityLink>> {
    let mut stmt = conn.prepare("SELECT id, source_entity_id, target_entity_id, type, description, weight, created_at, updated_at FROM links")?;
    let link_iter = stmt.query_map([], |row| {
        Ok(EntityLink {
            id: row.get(0)?,
            source_entity_id: row.get(1)?,
            target_entity_id: row.get(2)?,
            link_type: row.get(3)?,
            description: row.get(4)?,
            weight: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;

    let mut links = Vec::new();
    for link in link_iter {
        links.push(link?);
    }
    Ok(links)
}

pub fn delete_link(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM links WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn find_tag_by_name(conn: &Connection, name: &str) -> Result<Option<Tag>> {
    let mut stmt = conn.prepare("SELECT id, name, description, created_at, updated_at FROM tags WHERE name = ?1")?;
    let mut rows = stmt.query(params![name])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn find_entity_by_name(conn: &Connection, name: &str) -> Result<Option<Entity>> {
    let mut stmt = conn.prepare("SELECT id, name, type, description, properties, created_at, updated_at FROM entities WHERE LOWER(name) = LOWER(?1)")?;
    let mut rows = stmt.query(params![name])?;
    if let Some(row) = rows.next()? {
        let props_str: String = row.get(4)?;
        let properties: Value = serde_json::from_str(&props_str).unwrap_or(Value::Null);
        Ok(Some(Entity {
            id: row.get(0)?,
            name: row.get(1)?,
            entity_type: row.get(2)?,
            description: row.get(3)?,
            properties,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn link_exists(conn: &Connection, source_id: &str, target_id: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT id FROM links 
         WHERE (source_entity_id = ?1 AND target_entity_id = ?2)
            OR (source_entity_id = ?2 AND target_entity_id = ?1)"
    )?;
    let mut rows = stmt.query(params![source_id, target_id])?;
    if let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        Ok(Some(id))
    } else {
        Ok(None)
    }
}

pub fn increment_link_weight(conn: &Connection, link_id: &str, delta: f64, updated_at: &str) -> Result<()> {
    conn.execute(
        "UPDATE links SET weight = weight + ?1, updated_at = ?2 WHERE id = ?3",
        params![delta, updated_at, link_id],
    )?;
    Ok(())
}


// --- Flow Nodes ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FlowNode {
    pub id: String,
    pub title: String,
    pub content: Option<String>,
    pub x: f64,
    pub y: f64,
    pub conditions: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FlowLink {
    pub id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub label: Option<String>,
    pub conditions: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimelineEvent {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub event_date: String,
    pub associated_entity_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub fn create_flow_node(conn: &Connection, node: &FlowNode) -> Result<()> {
    conn.execute(
        "INSERT INTO flow_nodes (id, title, content, x, y, conditions, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![node.id, node.title, node.content, node.x, node.y, node.conditions, node.created_at, node.updated_at],
    )?;
    Ok(())
}

pub fn get_flow_nodes(conn: &Connection) -> Result<Vec<FlowNode>> {
    let mut stmt = conn.prepare("SELECT id, title, content, x, y, conditions, created_at, updated_at FROM flow_nodes")?;
    let node_iter = stmt.query_map([], |row| {
        Ok(FlowNode {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            x: row.get(3)?,
            y: row.get(4)?,
            conditions: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    let mut nodes = Vec::new();
    for n in node_iter {
        nodes.push(n?);
    }
    Ok(nodes)
}

pub fn update_flow_node_position(conn: &Connection, id: &str, x: f64, y: f64, updated_at: &str) -> Result<()> {
    conn.execute(
        "UPDATE flow_nodes SET x = ?1, y = ?2, updated_at = ?3 WHERE id = ?4",
        params![x, y, updated_at, id],
    )?;
    Ok(())
}

pub fn update_flow_node(conn: &Connection, node: &FlowNode) -> Result<()> {
    conn.execute(
        "UPDATE flow_nodes SET title = ?1, content = ?2, conditions = ?3, updated_at = ?4 WHERE id = ?5",
        params![node.title, node.content, node.conditions, node.updated_at, node.id],
    )?;
    Ok(())
}

pub fn delete_flow_node(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM flow_nodes WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn create_flow_link(conn: &Connection, link: &FlowLink) -> Result<()> {
    conn.execute(
        "INSERT INTO flow_links (id, source_node_id, target_node_id, label, conditions, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![link.id, link.source_node_id, link.target_node_id, link.label, link.conditions, link.created_at, link.updated_at],
    )?;
    Ok(())
}

pub fn get_flow_links(conn: &Connection) -> Result<Vec<FlowLink>> {
    let mut stmt = conn.prepare("SELECT id, source_node_id, target_node_id, label, conditions, created_at, updated_at FROM flow_links")?;
    let link_iter = stmt.query_map([], |row| {
        Ok(FlowLink {
            id: row.get(0)?,
            source_node_id: row.get(1)?,
            target_node_id: row.get(2)?,
            label: row.get(3)?,
            conditions: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    let mut links = Vec::new();
    for l in link_iter {
        links.push(l?);
    }
    Ok(links)
}

pub fn delete_flow_link(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM flow_links WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn create_timeline_event(conn: &Connection, ev: &TimelineEvent) -> Result<()> {
    conn.execute(
        "INSERT INTO timeline_events (id, title, description, event_date, associated_entity_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![ev.id, ev.title, ev.description, ev.event_date, ev.associated_entity_id, ev.created_at, ev.updated_at],
    )?;
    Ok(())
}

pub fn get_timeline_events(conn: &Connection) -> Result<Vec<TimelineEvent>> {
    let mut stmt = conn.prepare("SELECT id, title, description, event_date, associated_entity_id, created_at, updated_at FROM timeline_events ORDER BY event_date ASC")?;
    let ev_iter = stmt.query_map([], |row| {
        Ok(TimelineEvent {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            event_date: row.get(3)?,
            associated_entity_id: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    let mut events = Vec::new();
    for e in ev_iter {
        events.push(e?);
    }
    Ok(events)
}

pub fn delete_timeline_event(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM timeline_events WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn update_timeline_event(conn: &Connection, ev: &TimelineEvent) -> Result<()> {
    conn.execute(
        "UPDATE timeline_events SET title = ?1, description = ?2, event_date = ?3, associated_entity_id = ?4, updated_at = ?5 WHERE id = ?6",
        params![ev.title, ev.description, ev.event_date, ev.associated_entity_id, ev.updated_at, ev.id],
    )?;
    Ok(())
}


// ==========================================
// 4. Unit Tests Suite
// ==========================================


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        conn
    }

    #[test]
    fn test_project_info() {
        let conn = setup_test_db();
        let info = ProjectInfo {
            id: "proj_1".to_string(),
            name: "My Fantasy World".to_string(),
            description: Some("A world built on ancient steam machines".to_string()),
            created_at: "2026-06-08T00:00:00Z".to_string(),
            updated_at: "2026-06-08T00:00:00Z".to_string(),
        };

        save_project_info(&conn, &info).unwrap();

        let loaded = get_project_info(&conn).unwrap().unwrap();
        assert_eq!(loaded.id, "proj_1");
        assert_eq!(loaded.name, "My Fantasy World");
        assert_eq!(loaded.description.unwrap(), "A world built on ancient steam machines");
    }

    #[test]
    fn test_documents_crud() {
        let conn = setup_test_db();
        let doc = Document {
            id: "doc_1".to_string(),
            title: "Chapter 1: The Gathering".to_string(),
            relative_path: "chapters/chapter1.md".to_string(),
            content_type: "prose".to_string(),
            created_at: "2026-06-08T00:00:00Z".to_string(),
            updated_at: "2026-06-08T00:00:00Z".to_string(),
        };

        create_document(&conn, &doc).unwrap();

        let docs = get_documents(&conn).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].title, "Chapter 1: The Gathering");

        update_document(&conn, "doc_1", "Chapter 1: Renamed", "chapters/chapter1.md", "2026-06-08T01:00:00Z").unwrap();
        let docs = get_documents(&conn).unwrap();
        assert_eq!(docs[0].title, "Chapter 1: Renamed");

        delete_document(&conn, "doc_1").unwrap();
        let docs = get_documents(&conn).unwrap();
        assert_eq!(docs.len(), 0);
    }

    #[test]
    fn test_entities_crud() {
        let conn = setup_test_db();
        let entity = Entity {
            id: "ent_1".to_string(),
            name: "Arthur Pendragon".to_string(),
            entity_type: "character".to_string(),
            description: Some("The rightful King of Camelot".to_string()),
            properties: json!({
                "age": 25,
                "loyalty": "high",
                "traits": ["noble", "brave"]
            }),
            created_at: "2026-06-08T00:00:00Z".to_string(),
            updated_at: "2026-06-08T00:00:00Z".to_string(),
        };

        create_entity(&conn, &entity).unwrap();

        let entities = get_entities(&conn).unwrap();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].name, "Arthur Pendragon");
        assert_eq!(entities[0].properties["age"], 25);

        let updated_props = json!({
            "age": 26,
            "loyalty": "absolute"
        });
        update_entity(&conn, "ent_1", "King Arthur", Some("The King of Camelot"), &updated_props, "2026-06-08T01:00:00Z").unwrap();
        let entities = get_entities(&conn).unwrap();
        assert_eq!(entities[0].name, "King Arthur");
        assert_eq!(entities[0].properties["age"], 26);

        delete_entity(&conn, "ent_1").unwrap();
        let entities = get_entities(&conn).unwrap();
        assert_eq!(entities.len(), 0);
    }

    #[test]
    fn test_tags_and_associations() {
        let conn = setup_test_db();
        let tag = Tag {
            id: "tag_1".to_string(),
            name: "#Traicion".to_string(),
            description: Some("Acciones traicioneras".to_string()),
            created_at: "2026-06-08T00:00:00Z".to_string(),
            updated_at: "2026-06-08T00:00:00Z".to_string(),
        };
        create_tag(&conn, &tag).unwrap();

        let entity = Entity {
            id: "ent_1".to_string(),
            name: "Lancelot".to_string(),
            entity_type: "character".to_string(),
            description: Some("Knight of the Round Table".to_string()),
            properties: json!({}),
            created_at: "2026-06-08T00:00:00Z".to_string(),
            updated_at: "2026-06-08T00:00:00Z".to_string(),
        };
        create_entity(&conn, &entity).unwrap();

        associate_entity_tag(&conn, "ent_1", "tag_1").unwrap();

        let associated_tags = get_entity_tags(&conn, "ent_1").unwrap();
        assert_eq!(associated_tags.len(), 1);
        assert_eq!(associated_tags[0].name, "#Traicion");

        disassociate_entity_tag(&conn, "ent_1", "tag_1").unwrap();
        let associated_tags = get_entity_tags(&conn, "ent_1").unwrap();
        assert_eq!(associated_tags.len(), 0);
    }

    #[test]
    fn test_links_crud() {
        let conn = setup_test_db();
        let ent_a = Entity {
            id: "ent_a".to_string(),
            name: "Arthur".to_string(),
            entity_type: "character".to_string(),
            description: None,
            properties: json!({}),
            created_at: "2026-06-08T00:00:00Z".to_string(),
            updated_at: "2026-06-08T00:00:00Z".to_string(),
        };
        let ent_b = Entity {
            id: "ent_b".to_string(),
            name: "Guinevere".to_string(),
            entity_type: "character".to_string(),
            description: None,
            properties: json!({}),
            created_at: "2026-06-08T00:00:00Z".to_string(),
            updated_at: "2026-06-08T00:00:00Z".to_string(),
        };
        create_entity(&conn, &ent_a).unwrap();
        create_entity(&conn, &ent_b).unwrap();

        let link = EntityLink {
            id: "link_1".to_string(),
            source_entity_id: "ent_a".to_string(),
            target_entity_id: "ent_b".to_string(),
            link_type: "love".to_string(),
            description: Some("Spouses".to_string()),
            weight: 1.0,
            created_at: "2026-06-08T00:00:00Z".to_string(),
            updated_at: "2026-06-08T00:00:00Z".to_string(),
        };

        create_link(&conn, &link).unwrap();

        let links = get_links(&conn).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].link_type, "love");

        delete_link(&conn, "link_1").unwrap();
        let links = get_links(&conn).unwrap();
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn test_cascade_delete() {
        let conn = setup_test_db();
        let ent_a = Entity {
            id: "ent_a".to_string(),
            name: "Arthur".to_string(),
            entity_type: "character".to_string(),
            description: None,
            properties: json!({}),
            created_at: "2026-06-08T00:00:00Z".to_string(),
            updated_at: "2026-06-08T00:00:00Z".to_string(),
        };
        let ent_b = Entity {
            id: "ent_b".to_string(),
            name: "Guinevere".to_string(),
            entity_type: "character".to_string(),
            description: None,
            properties: json!({}),
            created_at: "2026-06-08T00:00:00Z".to_string(),
            updated_at: "2026-06-08T00:00:00Z".to_string(),
        };
        create_entity(&conn, &ent_a).unwrap();
        create_entity(&conn, &ent_b).unwrap();

        let link = EntityLink {
            id: "link_1".to_string(),
            source_entity_id: "ent_a".to_string(),
            target_entity_id: "ent_b".to_string(),
            link_type: "love".to_string(),
            description: Some("Spouses".to_string()),
            weight: 1.0,
            created_at: "2026-06-08T00:00:00Z".to_string(),
            updated_at: "2026-06-08T00:00:00Z".to_string(),
        };
        create_link(&conn, &link).unwrap();

        // Delete source entity, link should be deleted by cascade
        delete_entity(&conn, "ent_a").unwrap();

        let links = get_links(&conn).unwrap();
        assert_eq!(links.len(), 0); // Cascaded delete successful
    }
}
