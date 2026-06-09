import { invoke } from "@tauri-apps/api/core";

export interface ProjectInfo {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface Document {
  id: string;
  title: string;
  relative_path: string;
  content_type: "prose" | "screenplay";
  created_at: string;
  updated_at: string;
}

export interface Entity {
  id: string;
  name: string;
  entity_type: "character" | "location" | "item" | "faction" | "magic";
  description?: string;
  properties: any;
  created_at: string;
  updated_at: string;
}

export interface Tag {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface EntityLink {
  id: string;
  source_entity_id: string;
  target_entity_id: string;
  link_type: "family" | "affiliation" | "enmity" | "love" | "custom";
  description?: string;
  weight: number;
  created_at: string;
  updated_at: string;
}

// Command wrappers
export async function openProject(path: string): Promise<ProjectInfo> {
  return await invoke<ProjectInfo>("open_project", { path });
}

export async function getProjectInfo(): Promise<ProjectInfo> {
  return await invoke<ProjectInfo>("get_project_info");
}

export async function readDocumentFile(relativePath: string): Promise<string> {
  return await invoke<string>("read_document_file", { relativePath });
}

export async function writeDocumentFile(id: string, relativePath: string, content: string): Promise<void> {
  await invoke("write_document_file", { id, relativePath, content });
}

export async function createDocument(doc: Document): Promise<void> {
  await invoke("create_document", { doc });
}

export async function getDocuments(): Promise<Document[]> {
  return await invoke<Document[]>("get_documents");
}

export async function updateDocument(id: string, title: string, relative_path: string): Promise<void> {
  await invoke("update_document", { id, title, relative_path });
}

export async function deleteDocument(id: string): Promise<void> {
  await invoke("delete_document", { id });
}

export async function createEntity(entity: Entity): Promise<void> {
  await invoke("create_entity", { entity });
}

export async function getEntities(): Promise<Entity[]> {
  return await invoke<Entity[]>("get_entities");
}

export async function updateEntity(id: string, name: string, description: string | undefined, properties: any): Promise<void> {
  await invoke("update_entity", { id, name, description, properties });
}

export async function deleteEntity(id: string): Promise<void> {
  await invoke("delete_entity", { id });
}

export async function createTag(tag: Tag): Promise<void> {
  await invoke("create_tag", { tag });
}

export async function getTags(): Promise<Tag[]> {
  return await invoke<Tag[]>("get_tags");
}

export async function deleteTag(id: string): Promise<void> {
  await invoke("delete_tag", { id });
}

export async function associateEntityTag(entityId: string, tagId: string): Promise<void> {
  await invoke("associate_entity_tag", { entityId, tagId });
}

export async function disassociateEntityTag(entityId: string, tagId: string): Promise<void> {
  await invoke("disassociate_entity_tag", { entityId, tagId });
}

export async function getEntityTags(entityId: string): Promise<Tag[]> {
  return await invoke<Tag[]>("get_entity_tags", { entityId });
}

export async function associateDocumentTag(documentId: string, tagId: string): Promise<void> {
  await invoke("associate_document_tag", { documentId, tagId });
}

export async function disassociateDocumentTag(documentId: string, tagId: string): Promise<void> {
  await invoke("disassociate_document_tag", { documentId, tagId });
}

export async function getDocumentTags(documentId: string): Promise<Tag[]> {
  return await invoke<Tag[]>("get_document_tags", { documentId });
}

export async function createLink(link: EntityLink): Promise<void> {
  await invoke("create_link", { link });
}

export async function getLinks(): Promise<EntityLink[]> {
  return await invoke<EntityLink[]>("get_links");
}

export async function deleteLink(id: string): Promise<void> {
  await invoke("delete_link", { id });
}

export interface GraphNode {
  id: string;
  name: string;
  entity_type: string;
  connections_count: number;
  x?: number;
  y?: number;
  fx?: number | null;
  fy?: number | null;
}

export interface GraphLink {
  id: string;
  source: any;
  target: any;
  link_type: string;
  weight: number;
  description?: string;
}

export interface GraphData {
  nodes: GraphNode[];
  links: GraphLink[];
}

export async function getGraphData(): Promise<GraphData> {
  return await invoke<GraphData>("get_graph_data");
}

export interface FlowNode {
  id: string;
  title: string;
  content?: string;
  x: number;
  y: number;
  conditions?: string;
  created_at: string;
  updated_at: string;
}

export interface FlowLink {
  id: string;
  source_node_id: string;
  target_node_id: string;
  label?: string;
  conditions?: string;
  created_at: string;
  updated_at: string;
}

export interface TimelineEvent {
  id: string;
  title: string;
  description?: string;
  event_date: string;
  associated_entity_id?: string;
  created_at: string;
  updated_at: string;
}

// Flow Node commands
export async function createFlowNode(node: FlowNode): Promise<void> {
  await invoke("create_flow_node", { node });
}

export async function getFlowNodes(): Promise<FlowNode[]> {
  return await invoke<FlowNode[]>("get_flow_nodes");
}

export async function updateFlowNodePosition(id: string, x: number, y: number): Promise<void> {
  await invoke("update_flow_node_position", { id, x, y });
}

export async function updateFlowNode(node: FlowNode): Promise<void> {
  await invoke("update_flow_node", { node });
}

export async function deleteFlowNode(id: string): Promise<void> {
  await invoke("delete_flow_node", { id });
}

// Flow Link commands
export async function createFlowLink(link: FlowLink): Promise<void> {
  await invoke("create_flow_link", { link });
}

export async function getFlowLinks(): Promise<FlowLink[]> {
  return await invoke<FlowLink[]>("get_flow_links");
}

export async function deleteFlowLink(id: string): Promise<void> {
  await invoke("delete_flow_link", { id });
}

// Timeline Event commands
export async function createTimelineEvent(ev: TimelineEvent): Promise<void> {
  await invoke("create_timeline_event", { ev });
}

export async function getTimelineEvents(): Promise<TimelineEvent[]> {
  return await invoke<TimelineEvent[]>("get_timeline_events");
}

export async function updateTimelineEvent(ev: TimelineEvent): Promise<void> {
  await invoke("update_timeline_event", { ev });
}

export async function deleteTimelineEvent(id: string): Promise<void> {
  await invoke("delete_timeline_event", { id });
}

