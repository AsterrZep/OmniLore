<script lang="ts">
  import { onMount } from "svelte";
  import * as d3 from "d3";
  import {
    type ProjectInfo,
    type Document,
    type Entity,
    type Tag,
    type EntityLink,
    type GraphData,
    type GraphNode,
    type GraphLink,
    type FlowNode,
    type FlowLink,
    type TimelineEvent,
    openProject,
    getDocuments,
    createDocument,
    deleteDocument,
    getEntities,
    createEntity,
    deleteEntity,
    getTags,
    createTag,
    deleteTag,
    getLinks,
    createLink,
    deleteLink,
    readDocumentFile,
    writeDocumentFile,
    getGraphData,
    createFlowNode,
    getFlowNodes,
    updateFlowNodePosition,
    updateFlowNode,
    deleteFlowNode,
    createFlowLink,
    getFlowLinks,
    deleteFlowLink,
    createTimelineEvent,
    getTimelineEvents,
    updateTimelineEvent,
    deleteTimelineEvent
  } from "$lib/db";

  // App & View States
  let activeTab = $state("info");
  let projectPath = $state("/home/aster/omnilore_project_test");
  let errorMessage = $state("");
  let successMessage = $state("");
  let isDarkMode = $state(true);

  // Relational Data States
  let projectInfo = $state<ProjectInfo | null>(null);
  let documents = $state<Document[]>([]);
  let entities = $state<Entity[]>([]);
  let tags = $state<Tag[]>([]);
  let links = $state<EntityLink[]>([]);
  let flowNodes = $state<FlowNode[]>([]);
  let flowLinks = $state<FlowLink[]>([]);
  let timelineEvents = $state<TimelineEvent[]>([]);

  // --- Editor & Parsing States ---
  let selectedDoc = $state<Document | null>(null);
  let currentDocContent = $state("");
  let saveStatus = $state<"guardado" | "guardando" | "sin_guardar">("guardado");
  let editorMode = $state<"write" | "preview">("write");
  let zenMode = $state(false);
  let textareaElement = $state<HTMLTextAreaElement | null>(null);

  // Autocomplete states
  let showAutocomplete = $state(false);
  let autocompleteType = $state<"entity" | "tag" | null>(null);
  let autocompleteQuery = $state("");
  let autocompleteIndex = $state(0);
  let autocompletePosition = $state({ top: 0, left: 0 });

  // Live Analysis states
  let analysisEntities = $state<Entity[]>([]);
  let analysisTags = $state<Tag[]>([]);
  let unregisteredEntities = $state<string[]>([]);
  let unregisteredTags = $state<string[]>([]);
  let analysisSuggestedLinks = $state<{ source: Entity; target: Entity; existing: boolean }[]>([]);

  // Quick form helpers
  let quickEntType = $state<"character" | "location" | "item" | "faction" | "magic">("character");
  let quickTagDesc = $state("");

  // --- Graph Visualization States ---
  let lowPerformanceMode = $state(false);
  let graphData = $state<GraphData>({ nodes: [], links: [] });
  let isGraphLoading = $state(false);
  let svgElement = $state<SVGElement | null>(null);
  let canvasElement = $state<HTMLCanvasElement | null>(null);
  let simulation = $state<any>(null);
  let transform = $state({ x: 0, y: 0, k: 1 });
  let selectedNodeId = $state<string | null>(null);
  let hoveredNode = $state<GraphNode | null>(null);
  let tooltipPosition = $state({ x: 0, y: 0 });
  let entityTypeFilter = $state<string>("all");
  let searchGraphQuery = $state("");
  let draggedNode = $state<any>(null);

  // Form Input States
  let docTitle = $state("");
  let docPath = $state("");
  let docType = $state<"prose" | "screenplay">("prose");

  let entName = $state("");
  let entType = $state<"character" | "location" | "item" | "faction" | "magic">("character");
  let entDesc = $state("");
  let entAge = $state("");
  let entLoyalty = $state("high");

  let tagName = $state("");
  let tagDesc = $state("");

  let linkSource = $state("");
  let linkTarget = $state("");
  let linkType = $state("custom");
  let linkDesc = $state("");
  let linkWeight = $state(1.0);

  // Initialize Theme & Settings
  onMount(() => {
    // Default to dark mode on load
    document.documentElement.classList.add("dark-mode");
    
    // Check local storage for low performance mode
    const storedPerf = localStorage.getItem("omnilore_low_performance");
    if (storedPerf === "true") {
      lowPerformanceMode = true;
      document.documentElement.classList.add("low-perf-mode");
    }
  });

  function togglePerformanceMode() {
    lowPerformanceMode = !lowPerformanceMode;
    localStorage.setItem("omnilore_low_performance", lowPerformanceMode.toString());
    if (lowPerformanceMode) {
      document.documentElement.classList.add("low-perf-mode");
    } else {
      document.documentElement.classList.remove("low-perf-mode");
    }
    
    // Restart simulation to change engines if active
    if (activeTab === "graph") {
      loadGraphData();
    }
  }

  function toggleTheme() {
    isDarkMode = !isDarkMode;
    if (isDarkMode) {
      document.documentElement.classList.add("dark-mode");
    } else {
      document.documentElement.classList.remove("dark-mode");
    }
  }

  function showSuccess(msg: string) {
    successMessage = msg;
    errorMessage = "";
    setTimeout(() => { successMessage = ""; }, 3000);
  }

  function showError(msg: string) {
    errorMessage = msg;
    successMessage = "";
    setTimeout(() => { errorMessage = ""; }, 5000);
  }

  // Reload SQLite database contents
  async function refreshData() {
    if (!projectInfo) return;
    try {
      documents = await getDocuments();
      entities = await getEntities();
      tags = await getTags();
      links = await getLinks();
      flowNodes = await getFlowNodes();
      flowLinks = await getFlowLinks();
      timelineEvents = await getTimelineEvents();
    } catch (e: any) {
      showError("Error al refrescar datos: " + e.toString());
    }
  }

  // Project Open Handler
  async function handleOpenProject() {
    try {
      projectInfo = await openProject(projectPath);
      activeTab = "info";
      showSuccess(`Proyecto '${projectInfo.name}' cargado con éxito.`);
      await refreshData();
    } catch (e: any) {
      showError("Error al inicializar proyecto: " + e.toString());
      projectInfo = null;
    }
  }

  // --- Flow Visualizer States ---
  let flowPanX = $state(0);
  let flowPanY = $state(0);
  let flowZoom = $state(1.0);
  let isPanningFlow = $state(false);
  let panStartMouseX = 0;
  let panStartMouseY = 0;
  let panStartFlowX = 0;
  let panStartFlowY = 0;

  let draggedFlowNodeId = $state<string | null>(null);
  let nodeDragStartMouseX = 0;
  let nodeDragStartMouseY = 0;
  let nodeDragStartNodeX = 0;
  let nodeDragStartNodeY = 0;

  let linkingSourceNodeId = $state<string | null>(null);
  
  let selectedFlowNode = $state<FlowNode | null>(null);
  let selectedFlowNodeTitle = $state("");
  let selectedFlowNodeContent = $state("");
  let selectedFlowNodeConditions = $state("");

  // Flow node creation helpers
  let showCreateNodeModal = $state(false);
  let newFlowNodeTitle = $state("");
  let newFlowNodeContent = $state("");
  let newFlowNodeConditions = $state("");

  // Flow link creation helpers
  let showCreateLinkModal = $state(false);
  let targetLinkNodeId = $state("");
  let newFlowLinkLabel = $state("");
  let newFlowLinkConditions = $state("");

  // --- Timeline View States ---
  let selectedTimelineEvent = $state<TimelineEvent | null>(null);
  let timelineEventTitle = $state("");
  let timelineEventDescription = $state("");
  let timelineEventDate = $state("");
  let timelineEventEntityId = $state("");
  
  let showCreateTimelineEvent = $state(false);
  let newTimelineEventTitle = $state("");
  let newTimelineEventDescription = $state("");
  let newTimelineEventDate = $state("");
  let newTimelineEventEntityId = $state("");

  let timelineSortAsc = $state(true);
  let timelineSearchQuery = $state("");

  // Computed timeline events
  let filteredTimelineEvents = $derived(
    timelineEvents
      .filter(e => {
        if (!timelineSearchQuery) return true;
        const query = timelineSearchQuery.toLowerCase();
        return e.title.toLowerCase().includes(query) || 
               (e.description && e.description.toLowerCase().includes(query));
      })
      .sort((a, b) => {
        const comp = a.event_date.localeCompare(b.event_date, undefined, { numeric: true, sensitivity: 'base' });
        return timelineSortAsc ? comp : -comp;
      })
  );

  // Flow data handlers
  async function loadFlowData() {
    try {
      flowNodes = await getFlowNodes();
      flowLinks = await getFlowLinks();
    } catch (e: any) {
      showError("Error al cargar flujos: " + e.toString());
    }
  }

  async function handleCreateFlowNode() {
    try {
      const id = "node_" + Math.random().toString(36).substring(2, 11);
      // Center the node on canvas, account for pan
      const nodeX = 150 - flowPanX;
      const nodeY = 150 - flowPanY;
      const node: FlowNode = {
        id,
        title: newFlowNodeTitle || "Nuevo Nodo",
        content: newFlowNodeContent || undefined,
        x: nodeX,
        y: nodeY,
        conditions: newFlowNodeConditions || undefined,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      };
      await createFlowNode(node);
      showSuccess(`Nodo '${node.title}' creado.`);
      newFlowNodeTitle = "";
      newFlowNodeContent = "";
      newFlowNodeConditions = "";
      showCreateNodeModal = false;
      await refreshData();
    } catch (e: any) {
      showError("Error al crear nodo de flujo: " + e.toString());
    }
  }

  function startEditFlowNode(node: FlowNode) {
    selectedFlowNode = node;
    selectedFlowNodeTitle = node.title;
    selectedFlowNodeContent = node.content || "";
    selectedFlowNodeConditions = node.conditions || "";
  }

  async function handleUpdateFlowNode() {
    if (!selectedFlowNode) return;
    try {
      const updated: FlowNode = {
        ...selectedFlowNode,
        title: selectedFlowNodeTitle,
        content: selectedFlowNodeContent || undefined,
        conditions: selectedFlowNodeConditions || undefined,
        updated_at: new Date().toISOString()
      };
      await updateFlowNode(updated);
      showSuccess("Nodo de flujo actualizado.");
      selectedFlowNode = null;
      await refreshData();
    } catch (e: any) {
      showError("Error al actualizar nodo de flujo: " + e.toString());
    }
  }

  async function handleDeleteFlowNode(id: string) {
    if (!confirm("¿Seguro que deseas eliminar este nodo de flujo? Se eliminarán también las conexiones asociadas.")) return;
    try {
      await deleteFlowNode(id);
      showSuccess("Nodo de flujo eliminado.");
      if (selectedFlowNode && selectedFlowNode.id === id) {
        selectedFlowNode = null;
      }
      await refreshData();
    } catch (e: any) {
      showError("Error al eliminar nodo: " + e.toString());
    }
  }

  function startLinking(sourceId: string) {
    linkingSourceNodeId = sourceId;
    targetLinkNodeId = "";
    newFlowLinkLabel = "";
    newFlowLinkConditions = "";
    showCreateLinkModal = true;
  }

  async function handleCreateFlowLink() {
    if (!linkingSourceNodeId || !targetLinkNodeId) return;
    try {
      const id = "link_" + Math.random().toString(36).substring(2, 11);
      const link: FlowLink = {
        id,
        source_node_id: linkingSourceNodeId,
        target_node_id: targetLinkNodeId,
        label: newFlowLinkLabel || undefined,
        conditions: newFlowLinkConditions || undefined,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      };
      await createFlowLink(link);
      showSuccess("Conexión de flujo creada.");
      linkingSourceNodeId = null;
      targetLinkNodeId = "";
      newFlowLinkLabel = "";
      newFlowLinkConditions = "";
      showCreateLinkModal = false;
      await refreshData();
    } catch (e: any) {
      showError("Error al crear conexión: " + e.toString());
    }
  }

  async function handleDeleteFlowLink(id: string) {
    if (!confirm("¿Eliminar esta conexión de flujo?")) return;
    try {
      await deleteFlowLink(id);
      showSuccess("Conexión de flujo eliminada.");
      await refreshData();
    } catch (e: any) {
      showError("Error al eliminar conexión: " + e.toString());
    }
  }

  // Flow drag handlers
  function onNodeMouseDown(e: MouseEvent, id: string) {
    if (linkingSourceNodeId) return; // Don't drag if linking
    const target = e.target as HTMLElement;
    if (target.closest("button") || target.closest("input") || target.closest("textarea")) return;
    
    e.preventDefault();
    draggedFlowNodeId = id;
    const node = flowNodes.find(n => n.id === id);
    if (node) {
      nodeDragStartMouseX = e.clientX;
      nodeDragStartMouseY = e.clientY;
      nodeDragStartNodeX = node.x;
      nodeDragStartNodeY = node.y;
    }
  }

  function onCanvasMouseDown(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (target.closest(".flow-node-card") || target.closest(".modal-overlay") || target.closest(".editor-sidebar-panel")) return;
    
    e.preventDefault();
    isPanningFlow = true;
    panStartMouseX = e.clientX;
    panStartMouseY = e.clientY;
    panStartFlowX = flowPanX;
    panStartFlowY = flowPanY;
  }

  function onGlobalMouseMove(e: MouseEvent) {
    if (draggedFlowNodeId) {
      const dx = e.clientX - nodeDragStartMouseX;
      const dy = e.clientY - nodeDragStartMouseY;
      const node = flowNodes.find(n => n.id === draggedFlowNodeId);
      if (node) {
        node.x = nodeDragStartNodeX + dx;
        node.y = nodeDragStartNodeY + dy;
      }
    } else if (isPanningFlow) {
      flowPanX = panStartFlowX + (e.clientX - panStartMouseX);
      flowPanY = panStartFlowY + (e.clientY - panStartMouseY);
    }
  }

  async function onGlobalMouseUp() {
    if (draggedFlowNodeId) {
      const node = flowNodes.find(n => n.id === draggedFlowNodeId);
      if (node) {
        try {
          await updateFlowNodePosition(node.id, node.x, node.y);
        } catch (e: any) {
          console.error("Error updating node position:", e);
        }
      }
      draggedFlowNodeId = null;
    }
    isPanningFlow = false;
  }

  // Timeline Event handlers
  async function loadTimelineData() {
    try {
      timelineEvents = await getTimelineEvents();
    } catch (e: any) {
      showError("Error al cargar eventos de línea de tiempo: " + e.toString());
    }
  }

  async function handleCreateTimelineEvent() {
    try {
      const id = "ev_" + Math.random().toString(36).substring(2, 11);
      const ev: TimelineEvent = {
        id,
        title: newTimelineEventTitle || "Nuevo Evento",
        description: newTimelineEventDescription || undefined,
        event_date: newTimelineEventDate || new Date().toISOString().split("T")[0],
        associated_entity_id: newTimelineEventEntityId || undefined,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      };
      await createTimelineEvent(ev);
      showSuccess(`Evento '${ev.title}' creado.`);
      newTimelineEventTitle = "";
      newTimelineEventDescription = "";
      newTimelineEventDate = "";
      newTimelineEventEntityId = "";
      showCreateTimelineEvent = false;
      await refreshData();
    } catch (e: any) {
      showError("Error al crear evento: " + e.toString());
    }
  }

  function startEditTimelineEvent(ev: TimelineEvent) {
    selectedTimelineEvent = ev;
    timelineEventTitle = ev.title;
    timelineEventDescription = ev.description || "";
    timelineEventDate = ev.event_date;
    timelineEventEntityId = ev.associated_entity_id || "";
  }

  async function handleUpdateTimelineEvent() {
    if (!selectedTimelineEvent) return;
    try {
      const updated: TimelineEvent = {
        ...selectedTimelineEvent,
        title: timelineEventTitle,
        description: timelineEventDescription || undefined,
        event_date: timelineEventDate,
        associated_entity_id: timelineEventEntityId || undefined,
        updated_at: new Date().toISOString()
      };
      await updateTimelineEvent(updated);
      showSuccess("Evento de línea de tiempo actualizado.");
      selectedTimelineEvent = null;
      await refreshData();
    } catch (e: any) {
      showError("Error al actualizar evento: " + e.toString());
    }
  }

  async function handleDeleteTimelineEvent(id: string) {
    if (!confirm("¿Seguro que deseas eliminar este evento?")) return;
    try {
      await deleteTimelineEvent(id);
      showSuccess("Evento de línea de tiempo eliminado.");
      if (selectedTimelineEvent && selectedTimelineEvent.id === id) {
        selectedTimelineEvent = null;
      }
      await refreshData();
    } catch (e: any) {
      showError("Error al eliminar evento: " + e.toString());
    }
  }

  // --- Editor & Autocomplete Core Logic ---

  function cleanToken(token: string): string {
    let s = token.slice(1);
    while (s.length > 0) {
      const last = s[s.length - 1];
      if (/[a-zA-Z0-9_\u00C0-\u017F-]/.test(last)) {
        break;
      }
      s = s.slice(0, -1);
    }
    return s;
  }

  function runLiveAnalysis() {
    if (!currentDocContent) {
      analysisEntities = [];
      analysisTags = [];
      unregisteredEntities = [];
      unregisteredTags = [];
      analysisSuggestedLinks = [];
      return;
    }

    const words = currentDocContent.split(/\s+/);
    const foundEntitiesMap = new Map<string, Entity>();
    const foundTagsMap = new Map<string, Tag>();
    const potentialEntitiesSet = new Set<string>();
    const potentialTagsSet = new Set<string>();

    for (const word of words) {
      if (word.startsWith('@') && word.length > 1) {
        const cleaned = cleanToken(word);
        if (!cleaned) continue;
        const matched = entities.find(e => e.name.toLowerCase() === cleaned.toLowerCase());
        if (matched) {
          foundEntitiesMap.set(matched.id, matched);
        } else {
          potentialEntitiesSet.add(cleaned);
        }
      } else if (word.startsWith('#') && word.length > 1) {
        const cleaned = cleanToken(word);
        if (!cleaned) continue;
        const tagWithHash = '#' + cleaned;
        const matched = tags.find(t => t.name.toLowerCase() === tagWithHash.toLowerCase());
        if (matched) {
          foundTagsMap.set(matched.id, matched);
        } else {
          potentialTagsSet.add(cleaned);
        }
      }
    }

    analysisEntities = Array.from(foundEntitiesMap.values());
    analysisTags = Array.from(foundTagsMap.values());
    unregisteredEntities = Array.from(potentialEntitiesSet);
    unregisteredTags = Array.from(potentialTagsSet);

    // Predict link updates
    const detectedCharIds = analysisEntities
      .filter(e => e.entity_type === "character")
      .map(e => e.id);
      
    const suggested: typeof analysisSuggestedLinks = [];
    for (let i = 0; i < detectedCharIds.length; i++) {
      for (let j = i + 1; j < detectedCharIds.length; j++) {
        const charA = entities.find(e => e.id === detectedCharIds[i])!;
        const charB = entities.find(e => e.id === detectedCharIds[j])!;
        const exists = links.some(l => 
          (l.source_entity_id === charA.id && l.target_entity_id === charB.id) ||
          (l.source_entity_id === charB.id && l.target_entity_id === charA.id)
        );
        suggested.push({ source: charA, target: charB, existing: exists });
      }
    }
    analysisSuggestedLinks = suggested;
  }

  async function handleOpenEditor(doc: Document) {
    selectedDoc = doc;
    saveStatus = "guardado";
    editorMode = "write";
    showAutocomplete = false;
    try {
      currentDocContent = await readDocumentFile(doc.relative_path);
      runLiveAnalysis();
    } catch (e: any) {
      showError("Error al abrir archivo: " + e.toString());
      currentDocContent = "";
    }
  }

  async function handleSaveEditor() {
    if (!selectedDoc) return;
    saveStatus = "guardando";
    try {
      await writeDocumentFile(selectedDoc.id, selectedDoc.relative_path, currentDocContent);
      saveStatus = "guardado";
      await refreshData();
      runLiveAnalysis();
    } catch (e: any) {
      showError("Error al guardar: " + e.toString());
      saveStatus = "sin_guardar";
    }
  }

  let autoSaveTimer: any;
  function handleTextareaInput(e: Event) {
    saveStatus = "sin_guardar";
    clearTimeout(autoSaveTimer);
    autoSaveTimer = setTimeout(() => {
      handleSaveEditor();
    }, 1500);

    runLiveAnalysis();
    handleAutocompleteCheck();
  }

  // Derive filtered items dynamically using Svelte 5 $derived.by
  let filteredAutocompleteItems = $derived.by(() => {
    if (!showAutocomplete || !autocompleteType) return [];
    const q = autocompleteQuery.toLowerCase();
    if (autocompleteType === "entity") {
      return entities
        .filter(e => e.name.toLowerCase().includes(q))
        .map(e => ({ name: e.name, type: e.entity_type }));
    } else {
      return tags
        .filter(t => t.name.replace('#', '').toLowerCase().includes(q))
        .map(t => ({ name: t.name.replace('#', ''), type: 'tag' }));
    }
  });

  function handleAutocompleteCheck() {
    if (!textareaElement) return;
    const value = textareaElement.value;
    const start = textareaElement.selectionStart;
    
    let i = start - 1;
    while (i >= 0 && !/\s/.test(value[i])) {
      i--;
    }
    const word = value.substring(i + 1, start);
    
    if (word.startsWith('@')) {
      showAutocomplete = true;
      autocompleteType = "entity";
      autocompleteQuery = word.slice(1);
      autocompleteIndex = 0;
      updateAutocompletePosition();
    } else if (word.startsWith('#')) {
      showAutocomplete = true;
      autocompleteType = "tag";
      autocompleteQuery = word.slice(1);
      autocompleteIndex = 0;
      updateAutocompletePosition();
    } else {
      showAutocomplete = false;
      autocompleteType = null;
      autocompleteQuery = "";
    }
  }

  function updateAutocompletePosition() {
    if (!textareaElement) return;
    try {
      const coords = getCaretCoordinates(textareaElement);
      autocompletePosition = coords;
    } catch (err) {
      autocompletePosition = { top: 100, left: 100 };
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (showAutocomplete && filteredAutocompleteItems.length > 0) {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        autocompleteIndex = (autocompleteIndex + 1) % filteredAutocompleteItems.length;
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        autocompleteIndex = (autocompleteIndex - 1 + filteredAutocompleteItems.length) % filteredAutocompleteItems.length;
      } else if (e.key === "Enter" || e.key === "Tab") {
        e.preventDefault();
        selectAutocompleteItem(filteredAutocompleteItems[autocompleteIndex].name);
      } else if (e.key === "Escape") {
        e.preventDefault();
        showAutocomplete = false;
      }
    }
  }

  function selectAutocompleteItem(name: string) {
    if (!textareaElement) return;
    const start = textareaElement.selectionStart;
    const value = textareaElement.value;
    
    let i = start - 1;
    while (i >= 0 && !/\s/.test(value[i])) {
      i--;
    }
    const wordStart = i + 1;
    
    const prefix = value.substring(0, wordStart);
    const suffix = value.substring(start);
    
    const marker = autocompleteType === "entity" ? "@" : "#";
    const insertion = marker + name + " ";
    currentDocContent = prefix + insertion + suffix;
    showAutocomplete = false;
    
    setTimeout(() => {
      textareaElement?.focus();
      const pos = wordStart + insertion.length;
      textareaElement?.setSelectionRange(pos, pos);
      runLiveAnalysis();
      handleSaveEditor();
    }, 20);
  }

  async function handleQuickCreateEntity(name: string) {
    try {
      const entity: Entity = {
        id: crypto.randomUUID(),
        name: name,
        entity_type: quickEntType,
        description: "Creado automáticamente desde el editor",
        properties: { age: null, loyalty: "neutral" },
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      };
      await createEntity(entity);
      showSuccess(`Entidad @${name} registrada.`);
      await refreshData();
      runLiveAnalysis();
    } catch (err: any) {
      showError("Error al registrar: " + err.toString());
    }
  }

  async function handleQuickCreateTag(name: string) {
    const formatted = name.startsWith("#") ? name : "#" + name;
    try {
      const tag: Tag = {
        id: crypto.randomUUID(),
        name: formatted,
        description: quickTagDesc || "Palabra clave sugerida",
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      };
      await createTag(tag);
      quickTagDesc = "";
      showSuccess(`Palabra clave ${formatted} registrada.`);
      await refreshData();
      runLiveAnalysis();
    } catch (err: any) {
      showError("Error al registrar: " + err.toString());
    }
  }

  function getCaretCoordinates(textarea: HTMLTextAreaElement) {
    const selectionStart = textarea.selectionStart;
    const textBeforeCursor = textarea.value.substring(0, selectionStart);
    
    const mirror = document.createElement("div");
    const styles = window.getComputedStyle(textarea);
    
    for (let i = 0; i < styles.length; i++) {
      const key = styles[i];
      mirror.style.setProperty(key, styles.getPropertyValue(key));
    }
    
    mirror.style.position = "absolute";
    mirror.style.visibility = "hidden";
    mirror.style.whiteSpace = "pre-wrap";
    mirror.style.wordBreak = "break-all";
    mirror.style.overflow = "hidden";
    mirror.style.height = "auto";
    mirror.style.width = textarea.clientWidth + "px";
    
    mirror.textContent = textBeforeCursor;
    
    const span = document.createElement("span");
    span.textContent = "|";
    mirror.appendChild(span);
    
    textarea.parentElement?.appendChild(mirror);
    
    const top = span.offsetTop - textarea.scrollTop;
    const left = span.offsetLeft - textarea.scrollLeft;
    
    mirror.remove();
    
    return { 
      top: Math.min(top + 20, textarea.clientHeight - 120), 
      left: Math.min(left, textarea.clientWidth - 180) 
    };
  }

  function renderMarkdown(text: string): string {
    if (!text) return "<p class='empty-preview'>Comienza a escribir para ver la previsualización...</p>";
    
    let escaped = text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
      
    let html = escaped
      .replace(/^### (.*$)/gim, "<h3>$1</h3>")
      .replace(/^## (.*$)/gim, "<h2>$1</h2>")
      .replace(/^# (.*$)/gim, "<h1>$1</h1>")
      .replace(/\*\*(.*?)\*\*/g, "<strong>$1</strong>")
      .replace(/\*(.*?)\*/g, "<em>$1</em>")
      .replace(/_(.*?)_/g, "<em>$1</em>")
      .replace(/`([^`]+)`/g, "<code>$1</code>")
      .split(/\n\s*\n/)
      .map(p => {
        const trimmed = p.trim();
        if (trimmed.startsWith("<h") || trimmed.startsWith("<pre") || trimmed.startsWith("<code")) {
          return trimmed;
        }
        return `<p>${trimmed.replace(/\n/g, "<br>")}</p>`;
      })
      .join("\n");
      
    html = html.replace(/@([a-zA-Z0-9_-]+)/g, '<span class="fountain-mention">@$1</span>');
    html = html.replace(/#([a-zA-Z0-9_-]+)/g, '<span class="fountain-tag">#$1</span>');
    
    return html;
  }

  function renderFountain(text: string): string {
    if (!text) return "<p class='empty-preview'>Comienza a escribir para ver la previsualización del guion...</p>";
    
    let escaped = text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");

    const blocks = escaped.split(/\n\s*\n/);
    let html = "";
    
    for (let i = 0; i < blocks.length; i++) {
      const block = blocks[i].trim();
      if (!block) continue;
      
      const isSceneHeading = /^(INT\.|EXT\.|I\/E\.|INT\/EXT\.|EST\.)/i.test(block) || 
                             (/^[A-Z0-9_\-\s]+$/.test(block) && (block.startsWith("INT ") || block.startsWith("EXT ")));
      
      if (isSceneHeading) {
        html += `<div class="fountain-scene-heading">${block.toUpperCase()}</div>`;
        continue;
      }
      
      if (block.startsWith("&gt;") && block.endsWith("&lt;")) {
        const content = block.slice(4, -4).trim();
        html += `<div class="fountain-centered">${content}</div>`;
        continue;
      }
      if (block.startsWith("&gt;")) {
        const content = block.slice(4).trim();
        html += `<div class="fountain-transition">${content}</div>`;
        continue;
      }
      if (block.endsWith("TO:") && /^[A-Z\s]+$/.test(block)) {
        html += `<div class="fountain-transition">${block}</div>`;
        continue;
      }
      
      const lines = block.split("\n");
      const firstLine = lines[0].trim();
      
      const isCharacter = /^[A-Z0-9_\-\s@]+$/.test(firstLine) && lines.length === 1 && !firstLine.startsWith("INT.") && !firstLine.startsWith("EXT.");
      
      if (isCharacter && i + 1 < blocks.length) {
        let charName = firstLine;
        html += `<div class="fountain-character">${charName}</div>`;
        
        let nextBlock = blocks[i + 1].trim();
        i++;
        
        if (nextBlock.startsWith("(") && nextBlock.endsWith(")")) {
          html += `<div class="fountain-parenthetical">${nextBlock}</div>`;
          if (i + 1 < blocks.length) {
            let dialogueBlock = blocks[i + 1].trim();
            i++;
            html += `<div class="fountain-dialogue">${dialogueBlock.replace(/\n/g, "<br>")}</div>`;
          }
        } else {
          html += `<div class="fountain-dialogue">${nextBlock.replace(/\n/g, "<br>")}</div>`;
        }
        continue;
      }
      
      html += `<div class="fountain-action">${block.replace(/\n/g, "<br>")}</div>`;
    }
    
    html = html.replace(/@([a-zA-Z0-9_-]+)/g, '<span class="fountain-mention">@$1</span>');
    html = html.replace(/#([a-zA-Z0-9_-]+)/g, '<span class="fountain-tag">#$1</span>');
    
    return html;
  }

  // CRUD actions
  async function handleCreateDoc() {
    if (!docTitle || !docPath) {
      showError("Debes especificar el título y la ruta del archivo.");
      return;
    }
    try {
      const doc: Document = {
        id: crypto.randomUUID(),
        title: docTitle,
        relative_path: docPath,
        content_type: docType,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      };
      await createDocument(doc);
      docTitle = "";
      docPath = "";
      showSuccess("Documento creado en flat-files y base de datos.");
      await refreshData();
    } catch (e: any) {
      showError("Error al crear documento: " + e.toString());
    }
  }

  async function handleDeleteDoc(id: string) {
    try {
      await deleteDocument(id);
      showSuccess("Documento eliminado.");
      await refreshData();
    } catch (e: any) {
      showError("Error al eliminar documento: " + e.toString());
    }
  }

  async function handleCreateEntity() {
    if (!entName) {
      showError("El nombre de la entidad es obligatorio.");
      return;
    }
    try {
      const entity: Entity = {
        id: crypto.randomUUID(),
        name: entName,
        entity_type: entType,
        description: entDesc || undefined,
        properties: {
          age: entAge ? parseInt(entAge) : null,
          loyalty: entLoyalty
        },
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      };
      await createEntity(entity);
      entName = "";
      entDesc = "";
      entAge = "";
      showSuccess("Entidad agregada al constructor de mundos.");
      await refreshData();
    } catch (e: any) {
      showError("Error al guardar entidad: " + e.toString());
    }
  }

  async function handleDeleteEntity(id: string) {
    try {
      await deleteEntity(id);
      showSuccess("Entidad eliminada del mundo.");
      await refreshData();
    } catch (e: any) {
      showError("Error al eliminar entidad: " + e.toString());
    }
  }

  async function handleCreateTag() {
    if (!tagName) {
      showError("Ingresa el nombre de la palabra clave.");
      return;
    }
    const cleanTag = tagName.startsWith("#") ? tagName : "#" + tagName;
    try {
      const tag: Tag = {
        id: crypto.randomUUID(),
        name: cleanTag,
        description: tagDesc || undefined,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      };
      await createTag(tag);
      tagName = "";
      tagDesc = "";
      showSuccess("Palabra clave semántica creada.");
      await refreshData();
    } catch (e: any) {
      showError("Error al guardar etiqueta: " + e.toString());
    }
  }

  async function handleDeleteTag(id: string) {
    try {
      await deleteTag(id);
      showSuccess("Etiqueta temática eliminada.");
      await refreshData();
    } catch (e: any) {
      showError("Error al eliminar etiqueta: " + e.toString());
    }
  }

  async function handleCreateLink() {
    if (!linkSource || !linkTarget) {
      showError("Origen y Destino son obligatorios.");
      return;
    }
    try {
      const link: EntityLink = {
        id: crypto.randomUUID(),
        source_entity_id: linkSource,
        target_entity_id: linkTarget,
        link_type: linkType as any,
        description: linkDesc || undefined,
        weight: linkWeight,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      };
      await createLink(link);
      linkDesc = "";
      linkWeight = 1.0;
      showSuccess("Conexión agregada al grafo relacional.");
      await refreshData();
    } catch (e: any) {
      showError("Error al enlazar entidades: " + e.toString());
    }
  }

  async function handleDeleteLink(id: string) {
    try {
      await deleteLink(id);
      showSuccess("Conexión eliminada.");
      await refreshData();
    } catch (e: any) {
      showError("Error al eliminar enlace: " + e.toString());
    }
  }

  function getEntityName(id: string): string {
    const ent = entities.find(e => e.id === id);
    return ent ? ent.name : "Entidad eliminada";
  }

  // --- Graph Visualization Physics & Logic ---

  async function loadGraphData() {
    isGraphLoading = true;
    try {
      graphData = await getGraphData();
      setTimeout(() => {
        initSimulation();
      }, 50);
    } catch (e: any) {
      showError("Error al cargar grafo: " + e.toString());
    } finally {
      isGraphLoading = false;
    }
  }

  function nodesAreConnected(idA: string, idB: string): boolean {
    return graphData.links.some(l => {
      const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
      const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
      return (s === idA && t === idB) || (s === idB && t === idA);
    });
  }

  function isNodeVisible(node: any): boolean {
    const matchesType = entityTypeFilter === "all" || node.entity_type === entityTypeFilter;
    const matchesSearch = !searchGraphQuery || node.name.toLowerCase().includes(searchGraphQuery.toLowerCase());
    return matchesType && matchesSearch;
  }

  // Handle source/target as object or string (D3 maps source/target strings to node objects during compile)
  function isLinkVisible(link: any): boolean {
    const sId = typeof link.source === 'object' ? link.source.id : link.source;
    const tId = typeof link.target === 'object' ? link.target.id : link.target;
    const sNode = graphData.nodes.find(n => n.id === sId);
    const tNode = graphData.nodes.find(n => n.id === tId);
    if (!sNode || !tNode) return false;
    return isNodeVisible(sNode) && isNodeVisible(tNode);
  }

  function initSimulation() {
    if (simulation) simulation.stop();

    transform = { x: 0, y: 0, k: 1 };
    selectedNodeId = null;

    const nodes = graphData.nodes.map(n => ({ 
      ...n, 
      x: (Math.random() * 500 + 150), 
      y: (Math.random() * 350 + 100) 
    }));
    
    const links = graphData.links.map(l => ({ ...l }));

    simulation = d3.forceSimulation(nodes)
      .force("link", d3.forceLink(links).id((d: any) => d.id).distance((d: any) => {
        return Math.max(60, 200 - d.weight * 75);
      }))
      .force("charge", d3.forceManyBody().strength(-280))
      .force("center", d3.forceCenter(420, 280))
      .force("collision", d3.forceCollide().radius((d: any) => {
        return Math.max(25, d.connections_count * 2.5 + 16);
      }));

    if (lowPerformanceMode) {
      simulation.on("tick", () => {
        drawCanvas(nodes, links);
      });
      setupCanvasDrag(canvasElement, nodes);
    } else {
      simulation.on("tick", () => {
        graphData.nodes = [...nodes] as any;
        graphData.links = [...links] as any;
      });
      if (svgElement) {
        setupSVGZoom(svgElement);
      }
    }
  }

  function setupSVGZoom(svg: SVGElement) {
    const d3Svg = d3.select(svg);
    const zoomBehavior = d3.zoom()
      .scaleExtent([0.15, 6])
      .on("zoom", (event) => {
        transform = event.transform;
      });
    
    d3Svg.on(".zoom", null);
    d3Svg.call(zoomBehavior as any);
  }

  function setupCanvasDrag(canvas: HTMLCanvasElement | null, nodes: any[]) {
    if (!canvas) return;
    const d3Canvas = d3.select(canvas);
    
    d3Canvas.on(".drag", null);
    d3Canvas.on(".zoom", null);

    d3Canvas.call(d3.drag()
      .container(canvas)
      .subject((event) => {
        const mouseX = (event.x - transform.x) / transform.k;
        const mouseY = (event.y - transform.y) / transform.k;
        let closest: any = null;
        let minDist = 35;
        for (const node of nodes) {
          if (!isNodeVisible(node)) continue;
          const dx = node.x - mouseX;
          const dy = node.y - mouseY;
          const dist = Math.sqrt(dx*dx + dy*dy);
          if (dist < minDist) {
            minDist = dist;
            closest = node;
          }
        }
        return closest;
      })
      .on("start", (event) => {
        if (!event.active && simulation) simulation.alphaTarget(0.3).restart();
        event.subject.fx = event.subject.x;
        event.subject.fy = event.subject.y;
        selectedNodeId = event.subject.id;
      })
      .on("drag", (event) => {
        event.subject.fx = (event.x - transform.x) / transform.k;
        event.subject.fy = (event.y - transform.y) / transform.k;
      })
      .on("end", (event) => {
        if (!event.active && simulation) simulation.alphaTarget(0);
        event.subject.fx = null;
        event.subject.fy = null;
      }) as any
    );

    d3Canvas.call(d3.zoom()
      .scaleExtent([0.15, 6])
      .on("zoom", (event) => {
        transform = event.transform;
        drawCanvas(nodes, graphData.links);
      }) as any
    );
  }

  function drawCanvas(nodes: any[], links: any[]) {
    if (!canvasElement) return;
    const ctx = canvasElement.getContext("2d");
    if (!ctx) return;

    ctx.clearRect(0, 0, canvasElement.width, canvasElement.height);

    ctx.save();
    ctx.translate(transform.x, transform.y);
    ctx.scale(transform.k, transform.k);

    ctx.lineWidth = 1.5;
    for (const link of links) {
      if (!isLinkVisible(link)) continue;
      
      const sId = typeof link.source === 'object' ? link.source.id : link.source;
      const tId = typeof link.target === 'object' ? link.target.id : link.target;
      
      const sourceNode = nodes.find(n => n.id === sId);
      const targetNode = nodes.find(n => n.id === tId);
      if (!sourceNode || !targetNode) continue;

      const isHighlighted = selectedNodeId === sourceNode.id || selectedNodeId === targetNode.id;
      const opacity = selectedNodeId ? (isHighlighted ? 0.9 : 0.1) : 0.35;
      
      ctx.strokeStyle = isHighlighted ? "#007aff" : "rgba(180, 180, 180, " + opacity + ")";
      ctx.beginPath();
      ctx.moveTo(sourceNode.x, sourceNode.y);
      ctx.lineTo(targetNode.x, targetNode.y);
      ctx.stroke();
    }

    for (const node of nodes) {
      if (!isNodeVisible(node)) continue;
      
      const radius = Math.max(9, node.connections_count * 1.5 + 7);
      
      let color = "#007aff";
      if (node.entity_type === "location") color = "#34c759";
      else if (node.entity_type === "item") color = "#ff9500";
      else if (node.entity_type === "faction") color = "#5856d6";
      else if (node.entity_type === "magic") color = "#ff3b30";

      const isSelected = selectedNodeId === node.id;
      const isLinkedToSelected = selectedNodeId ? nodesAreConnected(selectedNodeId, node.id) : false;
      
      const opacity = selectedNodeId ? (isSelected || isLinkedToSelected ? 1.0 : 0.15) : 0.85;

      ctx.fillStyle = color;
      ctx.globalAlpha = opacity;
      ctx.beginPath();
      ctx.arc(node.x, node.y, radius, 0, 2 * Math.PI);
      ctx.fill();

      if (isSelected) {
        ctx.strokeStyle = "#ffffff";
        ctx.lineWidth = 2.5;
        ctx.stroke();
      }

      ctx.fillStyle = isSelected ? "#ffffff" : "var(--text-primary)";
      ctx.font = isSelected ? "bold 11px sans-serif" : "9px sans-serif";
      ctx.fillText(node.name, node.x + radius + 4, node.y + 3);
      ctx.globalAlpha = 1.0;
    }

    ctx.restore();
  }

  // --- SVG Drag & Drop ---
  function handleNodeDragStart(e: MouseEvent, node: any) {
    if (!simulation) return;
    draggedNode = node;
    node.fx = node.x;
    node.fy = node.y;
    simulation.alphaTarget(0.3).restart();
  }

  function handleNodeDrag(e: MouseEvent) {
    if (!draggedNode || !svgElement) return;
    const rect = svgElement.getBoundingClientRect();
    const mouseX = (e.clientX - rect.left - transform.x) / transform.k;
    const mouseY = (e.clientY - rect.top - transform.y) / transform.k;
    draggedNode.fx = mouseX;
    draggedNode.fy = mouseY;
  }

  function handleNodeDragEnd() {
    if (!draggedNode) return;
    if (simulation) simulation.alphaTarget(0);
    draggedNode.fx = null;
    draggedNode.fy = null;
    draggedNode = null;
  }

  function handleNodeClick(node: GraphNode) {
    if (selectedNodeId === node.id) {
      selectedNodeId = null;
    } else {
      selectedNodeId = node.id;
    }
  }

  function handleNodeMouseEnter(e: MouseEvent, node: GraphNode) {
    hoveredNode = node;
    const rect = svgElement?.getBoundingClientRect();
    if (rect) {
      tooltipPosition = {
        x: e.clientX - rect.left + 15,
        y: e.clientY - rect.top + 15
      };
    }
  }

  function handleNodeMouseLeave() {
    hoveredNode = null;
  }
</script>

<svelte:window onmousemove={onGlobalMouseMove} onmouseup={onGlobalMouseUp} />

<div class="window-shell">
  <!-- macOS Window Decoration Header -->
  <header class="window-header glass">
    <div class="window-controls">
      <span class="dot red"></span>
      <span class="dot yellow"></span>
      <span class="dot green"></span>
    </div>
    
    <div class="title-bar">
      <span class="app-title">OmniLore</span>
      {#if projectInfo}
        <span class="separator">/</span>
        <span class="project-title">{projectInfo.name}</span>
      {/if}
    </div>

    <div class="window-actions">
      <!-- Dark/Light theme toggle -->
      <button class="theme-toggle clickable" onclick={toggleTheme} title="Cambiar Tema">
        {#if isDarkMode}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="theme-icon"><circle cx="12" cy="12" r="5"></circle><line x1="12" y1="1" x2="12" y2="3"></line><line x1="12" y1="21" x2="12" y2="23"></line><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line><line x1="1" y1="12" x2="3" y2="12"></line><line x1="21" y1="12" x2="23" y2="12"></line><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line></svg>
        {:else}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="theme-icon"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path></svg>
        {/if}
      </button>
    </div>
  </header>

  <!-- Alerts -->
  {#if errorMessage}
    <div class="alert alert-error">{errorMessage}</div>
  {/if}
  {#if successMessage}
    <div class="alert alert-success">{successMessage}</div>
  {/if}

  <!-- Project Path Initializer (if no project loaded) -->
  {#if !projectInfo}
    <div class="project-loader">
      <div class="loader-card glass">
        <div class="branding-logo">
          <svg class="brand-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"></path>
          </svg>
        </div>
        <h2>Cargar Espacio de Trabajo</h2>
        <p>Selecciona o escribe el directorio local para almacenar los textos planos (.md/.fountain) y la base de datos SQLite.</p>
        <div class="loader-input-group">
          <input type="text" placeholder="Ruta absoluta de carpeta..." bind:value={projectPath} />
          <button class="btn btn-primary" onclick={handleOpenProject}>Cargar Proyecto</button>
        </div>
      </div>
    </div>
  {:else}
    <!-- Main Workspace -->
    <div class="workspace-layout">
      
      <!-- macOS-style Sidebar -->
      <aside class="sidebar glass">
        <nav class="nav-menu">
          <button class="nav-btn" class:active={activeTab === "info"} onclick={() => activeTab = "info"}>
            <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><line x1="9" y1="3" x2="9" y2="21"></line></svg>
            <span>Detalles</span>
          </button>
          
          <div class="nav-section-title">Escritura</div>
          <button class="nav-btn" class:active={activeTab === "docs"} onclick={() => activeTab = "docs"}>
            <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg>
            <span>Documentos</span>
          </button>

          <div class="nav-section-title">Worldbuilding</div>
          <button class="nav-btn" class:active={activeTab === "entities"} onclick={() => activeTab = "entities"}>
            <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path><circle cx="12" cy="7" r="4"></circle></svg>
            <span>Entidades</span>
          </button>
          <button class="nav-btn" class:active={activeTab === "tags"} onclick={() => activeTab = "tags"}>
            <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path><line x1="7" y1="7" x2="7.01" y2="7"></line></svg>
            <span>Etiquetas</span>
          </button>
          <button class="nav-btn" class:active={activeTab === "links"} onclick={() => activeTab = "links"}>
            <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path></svg>
            <span>Relaciones</span>
          </button>
          <button class="nav-btn" class:active={activeTab === "graph"} onclick={() => { activeTab = "graph"; loadGraphData(); }}>
            <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="18" cy="5" r="3"></circle><circle cx="6" cy="12" r="3"></circle><circle cx="18" cy="19" r="3"></circle><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"></line><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"></line></svg>
            <span>Grafo 2D</span>
          </button>
          <button class="nav-btn" class:active={activeTab === "flow"} onclick={() => { activeTab = "flow"; loadFlowData(); }}>
            <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="9" rx="1"></rect><rect x="14" y="3" width="7" height="5" rx="1"></rect><rect x="14" y="12" width="7" height="9" rx="1"></rect><rect x="3" y="16" width="7" height="5" rx="1"></rect><path d="M7 12v4M14 5.5H10v11h4M7 8h3"></path></svg>
            <span>Flujos</span>
          </button>
          <button class="nav-btn" class:active={activeTab === "timeline"} onclick={() => { activeTab = "timeline"; loadTimelineData(); }}>
            <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>
            <span>Línea Temporal</span>
          </button>
        </nav>

        <!-- Sidebar footer project path change -->
        <div class="sidebar-footer">
          <div class="path-display" title={projectPath}>
            <span class="label">Carpeta:</span>
            <span class="value">{projectPath.split("/").pop()}</span>
          </div>
          <button class="btn btn-secondary btn-sm full-width" onclick={() => projectInfo = null}>Cerrar</button>
        </div>
      </aside>

      <!-- Main Panels Panel -->
      <main class="panel-content">
        {#if activeTab === "info"}
          <div class="card glass animate-fade-in">
            <h2>Metadatos del Proyecto</h2>
            <div class="apple-table">
              <div class="table-row">
                <span class="col-lbl">Identificador (UUID)</span>
                <span class="col-val code">{projectInfo.id}</span>
              </div>
              <div class="table-row">
                <span class="col-lbl">Nombre del Proyecto</span>
                <span class="col-val">{projectInfo.name}</span>
              </div>
              <div class="table-row">
                <span class="col-lbl">Descripción</span>
                <span class="col-val">{projectInfo.description || "Sin descripción registrada"}</span>
              </div>
              <div class="table-row">
                <span class="col-lbl">Fecha de Creación</span>
                <span class="col-val">{new Date(projectInfo.created_at).toLocaleString()}</span>
              </div>
              <div class="table-row">
                <span class="col-lbl">Base de Datos Local</span>
                <span class="col-val code">{projectPath}/project.db</span>
              </div>
            </div>
          </div>

          <!-- Performance settings panel -->
          <div class="card glass animate-fade-in" style="margin-top: var(--space-4);">
            <h2>Ajustes de Rendimiento y GPU</h2>
            <p style="color: var(--text-secondary); font-size: 13.5px; margin-bottom: var(--space-4); line-height: 1.4;">
              Si experimentas lentitud o parpadeos (debido a la ejecución en contenedores de ChromeOS o sistemas de bajo recurso sin aceleración de GPU nativa), puedes desactivar los efectos visuales complejos y optimizar los gráficos.
            </p>
            <div class="apple-table">
              <div class="table-row">
                <span class="col-lbl">Efectos Premium (Sombras, Blur, Transiciones)</span>
                <span class="col-val">
                  <button class="btn {lowPerformanceMode ? 'btn-secondary' : 'btn-primary'} btn-sm" onclick={togglePerformanceMode}>
                    {lowPerformanceMode ? "✕ Desactivados (Modo CPU Lenta)" : "✓ Activados (Modo GPU)"}
                  </button>
                </span>
              </div>
              <div class="table-row">
                <span class="col-lbl">Motor del Grafo 2D</span>
                <span class="col-val" style="font-weight: 600;">
                  {lowPerformanceMode ? "🎨 Canvas 2D (Optimizado para CPU)" : "📐 Vectorial SVG (Consumo GPU)"}
                </span>
              </div>
            </div>
          </div>

        {:else if activeTab === "docs"}
          {#if selectedDoc}
            <div class="editor-workspace" class:zen={zenMode}>
              <!-- Header / Top Bar -->
              <div class="editor-header">
                <div class="header-left">
                  <button class="btn btn-secondary btn-sm" onclick={() => { selectedDoc = null; refreshData(); }}>
                    ➔ Volver
                  </button>
                  <span class="editor-doc-title">{selectedDoc.title}</span>
                  <span class="badge {selectedDoc.content_type}">{selectedDoc.content_type}</span>
                </div>
                
                <div class="header-center">
                  <button class="tab-btn" class:active={editorMode === "write"} onclick={() => editorMode = "write"}>Editar</button>
                  <button class="tab-btn" class:active={editorMode === "preview"} onclick={() => editorMode = "preview"}>Vista Previa</button>
                </div>

                <div class="header-right">
                  <span class="save-indicator {saveStatus}">
                    {#if saveStatus === "guardando"}
                      <span class="spinner"></span> Guardando...
                    {:else if saveStatus === "sin_guardar"}
                      ✍ Editando...
                    {:else}
                      ✓ Guardado
                    {/if}
                  </span>
                  <button class="btn btn-secondary btn-sm" style="margin-left: 8px;" onclick={() => zenMode = !zenMode}>
                    {zenMode ? "Mostrar Interfaz" : "Modo Zen"}
                  </button>
                </div>
              </div>

              <!-- Main Split Area: Editor Canvas + Live Sidebar -->
              <div class="editor-container">
                <!-- Editor Canvas -->
                <div class="editor-canvas-wrapper">
                  {#if editorMode === "write"}
                    <div class="textarea-relative-container">
                      <textarea
                        bind:this={textareaElement}
                        class="editor-textarea {selectedDoc.content_type}"
                        placeholder={selectedDoc.content_type === "screenplay" ? "Escribe tu guion en formato Fountain (ej. INT. BOSQUE - DÍA)..." : "Comienza a escribir tu novela en prosa..."}
                        bind:value={currentDocContent}
                        oninput={handleTextareaInput}
                        onkeydown={handleKeydown}
                      ></textarea>

                      <!-- Floating Autocomplete dropdown -->
                      {#if showAutocomplete && filteredAutocompleteItems.length > 0}
                        <div class="autocomplete-dropdown glass" style="top: {autocompletePosition.top}px; left: {autocompletePosition.left}px;">
                          {#each filteredAutocompleteItems as item, idx}
                            <button
                              class="autocomplete-item"
                              class:selected={idx === autocompleteIndex}
                              onclick={() => selectAutocompleteItem(item.name)}
                            >
                              <span class="ac-symbol">{autocompleteType === "entity" ? "@" : "#"}</span>
                              <span class="ac-name">{item.name}</span>
                              {#if item.type !== "tag"}
                                <span class="ac-type">{item.type}</span>
                              {/if}
                            </button>
                          {/each}
                        </div>
                      {/if}
                    </div>
                  {:else}
                    <!-- Preview Mode -->
                    <div class="preview-container">
                      {#if selectedDoc.content_type === "screenplay"}
                        <div class="screenplay-paper">
                          {@html renderFountain(currentDocContent)}
                        </div>
                      {:else}
                        <div class="prose-paper">
                          {@html renderMarkdown(currentDocContent)}
                        </div>
                      {/if}
                    </div>
                  {/if}
                </div>

                <!-- Collapsible Right Sidebar (Analysis Panel) -->
                {#if !zenMode}
                  <aside class="analysis-sidebar glass">
                    <div class="analysis-section">
                      <h3>Análisis de Escena</h3>
                      <div class="stats-row">
                        <div class="stat-card">
                          <span class="stat-num">{analysisEntities.length}</span>
                          <span class="stat-lbl">Entidades</span>
                        </div>
                        <div class="stat-card">
                          <span class="stat-num">{analysisTags.length}</span>
                          <span class="stat-lbl">Etiquetas</span>
                        </div>
                      </div>
                    </div>

                    <!-- Registered mentions -->
                    <div class="analysis-section">
                      <h4>Entidades de la Lore</h4>
                      {#if analysisEntities.length === 0}
                        <p class="section-empty">Menciona personajes con @ en el texto.</p>
                      {:else}
                        <div class="pill-cloud">
                          {#each analysisEntities as ent}
                            <span class="entity-pill {ent.entity_type}">@{ent.name}</span>
                          {/each}
                        </div>
                      {/if}
                    </div>

                    <!-- Registered tags -->
                    <div class="analysis-section">
                      <h4>Temas Asociados</h4>
                      {#if analysisTags.length === 0}
                        <p class="section-empty">Menciona temas con # en el texto.</p>
                      {:else}
                        <div class="pill-cloud">
                          {#each analysisTags as tag}
                            <span class="tag-pill">{tag.name}</span>
                          {/each}
                        </div>
                      {/if}
                    </div>

                    <!-- Co-occurrence relation predictions -->
                    <div class="analysis-section">
                      <h4>Atracción de Personajes (Co-ocurrencias)</h4>
                      {#if analysisSuggestedLinks.length === 0}
                        <p class="section-empty">Ningún par de personajes detectado.</p>
                      {:else}
                        <div class="relation-predictions">
                          {#each analysisSuggestedLinks as rel}
                            <div class="relation-prediction-item">
                              <span class="rel-node">@{rel.source.name}</span>
                              <span class="rel-connector">🔗</span>
                              <span class="rel-node">@{rel.target.name}</span>
                              {#if rel.existing}
                                <span class="rel-status badge character">Fuerza +0.1</span>
                              {:else}
                                <span class="rel-status badge magic">Crear Conexión</span>
                              {/if}
                            </div>
                          {/each}
                        </div>
                      {/if}
                    </div>

                    <!-- Unregistered Entities / Tags Suggestions with Quick Add buttons -->
                    {#if unregisteredEntities.length > 0 || unregisteredTags.length > 0}
                      <div class="analysis-section suggestions-section">
                        <h4>Propuestas de Registro</h4>
                        <p class="section-subtitle">Hemos encontrado palabras clave o menciones no registradas en la Wiki. Añádelas con un clic:</p>
                        
                        {#each unregisteredEntities as uent}
                          <div class="suggestion-item">
                            <span class="suggestion-name">@{uent}</span>
                            <div class="suggestion-actions">
                              <select class="select-xs" bind:value={quickEntType}>
                                <option value="character">Personaje</option>
                                <option value="location">Ubicación</option>
                                <option value="item">Objeto</option>
                                <option value="faction">Alianza</option>
                                <option value="magic">Magia</option>
                              </select>
                              <button class="btn btn-primary btn-xs" onclick={() => handleQuickCreateEntity(uent)}>+</button>
                            </div>
                          </div>
                        {/each}

                        {#each unregisteredTags as utag}
                          <div class="suggestion-item">
                            <span class="suggestion-name">#{utag}</span>
                            <div class="suggestion-actions">
                              <input class="input-xs" type="text" placeholder="Descripción breve..." bind:value={quickTagDesc} />
                              <button class="btn btn-primary btn-xs" onclick={() => handleQuickCreateTag(utag)}>+</button>
                            </div>
                          </div>
                        {/each}
                      </div>
                    {/if}
                  </aside>
                {/if}
              </div>
            </div>
          {:else}
            <div class="split-pane animate-fade-in">
              <!-- List of documents -->
              <div class="card glass pane-list">
                <h2>Archivos del Estudio</h2>
                {#if documents.length === 0}
                  <div class="empty-state">No hay documentos de prosa o guion creados.</div>
                {:else}
                  <div class="apple-list">
                    {#each documents as doc}
                      <div class="apple-item">
                        <div class="item-details">
                          <div class="item-primary">
                            <span class="doc-icon">
                              {#if doc.content_type === "screenplay"}🎬{:else}📖{/if}
                            </span>
                            {doc.title}
                          </div>
                          <div class="item-secondary">Ruta: {doc.relative_path} | Formato: {doc.content_type}</div>
                        </div>
                        <div class="item-actions">
                          <button class="btn btn-primary btn-sm" style="margin-right: 8px;" onclick={() => handleOpenEditor(doc)}>Escribir</button>
                          <button class="btn btn-danger btn-sm" onclick={() => handleDeleteDoc(doc.id)}>Borrar</button>
                        </div>
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>

              <!-- Creation form -->
              <div class="card glass pane-form">
                <h2>Nuevo Documento</h2>
                <div class="form-group">
                  <label for="doc-title">Nombre del Documento</label>
                  <input id="doc-title" type="text" placeholder="Ej. Capítulo 1: El Inicio" bind:value={docTitle} />
                </div>
                <div class="form-group">
                  <label for="doc-path">Ruta de Archivo</label>
                  <input id="doc-path" type="text" placeholder="Ej. chapters/cap1.md" bind:value={docPath} />
                </div>
                <div class="form-group">
                  <label for="doc-type">Formato de Formato</label>
                  <select id="doc-type" bind:value={docType}>
                    <option value="prose">Novela (Prosa Markdown .md)</option>
                    <option value="screenplay">Guion Audiovisual (Fountain .fountain)</option>
                  </select>
                </div>
                <button class="btn btn-primary full-width margin-top-md" onclick={handleCreateDoc}>Añadir Archivo</button>
              </div>
            </div>
          {/if}

        {:else if activeTab === "graph"}
          <div class="card glass animate-fade-in" style="height: 100%; display: flex; flex-direction: column; overflow: hidden; padding: var(--space-4);">
            <!-- Graph controls header -->
            <div class="graph-header" style="display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-3); flex-wrap: wrap; gap: 12px;">
              <div>
                <h2 style="margin-bottom: 2px;">Grafo Relacional del Universo</h2>
                <span class="item-secondary">Gravedad e Interacciones Físicas de tu Lore</span>
              </div>
              <div style="display: flex; gap: 8px; align-items: center;">
                <input type="text" placeholder="Buscar entidad..." style="width: 150px; height: 32px;" bind:value={searchGraphQuery} oninput={initSimulation} />
                <select style="width: 130px; height: 32px;" bind:value={entityTypeFilter} onchange={initSimulation}>
                  <option value="all">Ver Todos</option>
                  <option value="character">Personajes</option>
                  <option value="location">Ubicaciones</option>
                  <option value="item">Objetos</option>
                  <option value="faction">Alianzas</option>
                  <option value="magic">Magia</option>
                </select>
                <button class="btn btn-secondary btn-sm" style="height: 32px;" onclick={initSimulation}>Reiniciar Grafo</button>
              </div>
            </div>

            <!-- Graph Canvas / SVG Container -->
            <div class="graph-canvas-container" style="flex-grow: 1; border: 1px solid var(--border-primary); border-radius: var(--radius-md); background-color: var(--bg-system); position: relative; overflow: hidden; height: 500px;">
              {#if isGraphLoading}
                <div class="empty-state" style="position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; flex-direction: column;">
                  <span class="spinner" style="width: 24px; height: 24px;"></span>
                  <p style="margin-top: var(--space-2);">Calculando distancias del multiverso...</p>
                </div>
              {:else if graphData.nodes.length === 0}
                <div class="empty-state" style="position: absolute; inset: 0; display: flex; align-items: center; justify-content: center;">
                  Carga o crea entidades y relaciones para comenzar a renderizar el grafo.
                </div>
              {:else}
                <!-- Low-perf Canvas engine -->
                {#if lowPerformanceMode}
                  <canvas 
                    bind:this={canvasElement} 
                    width="840" 
                    height="500" 
                    class="graph-canvas"
                    style="width: 100%; height: 100%; display: block;"
                  ></canvas>
                {:else}
                  <!-- High-perf SVG engine -->
                  <svg 
                    bind:this={svgElement} 
                    class="graph-svg" 
                    width="100%" 
                    height="100%"
                    style="display: block; cursor: grab;"
                    onmousemove={handleNodeDrag}
                    onmouseup={handleNodeDragEnd}
                    onmouseleave={handleNodeDragEnd}
                    role="application"
                    aria-label="Grafo Relacional de Lore"
                  >
                    <!-- Zoomable group -->
                    <g transform="translate({transform.x}, {transform.y}) scale({transform.k})">
                      
                      <!-- Links -->
                      {#each graphData.links as link}
                        {#if isLinkVisible(link)}
                          {@const sId = typeof link.source === 'object' ? link.source.id : link.source}
                          {@const tId = typeof link.target === 'object' ? link.target.id : link.target}
                          {@const sNode = graphData.nodes.find(n => n.id === sId)}
                          {@const tNode = graphData.nodes.find(n => n.id === tId)}
                          {#if sNode && tNode}
                            {@const isHighlighted = selectedNodeId === sId || selectedNodeId === tId}
                            {@const opacity = selectedNodeId ? (isHighlighted ? 0.9 : 0.08) : 0.4}
                            
                            <line
                              x1={sNode.x}
                              y1={sNode.y}
                              x2={tNode.x}
                              y2={tNode.y}
                              stroke={isHighlighted ? "var(--accent)" : "var(--text-tertiary)"}
                              stroke-width={isHighlighted ? 2.5 : 1.5}
                              stroke-opacity={opacity}
                            />
                          {/if}
                        {/if}
                      {/each}

                      <!-- Nodes -->
                      {#each graphData.nodes as node}
                        {#if isNodeVisible(node)}
                          {@const radius = Math.max(9, node.connections_count * 1.5 + 7)}
                          {@const isSelected = selectedNodeId === node.id}
                          {@const isLinkedToSelected = selectedNodeId ? nodesAreConnected(selectedNodeId, node.id) : false}
                          {@const opacity = selectedNodeId ? (isSelected || isLinkedToSelected ? 1.0 : 0.15) : 0.9}

                          <g 
                            transform="translate({node.x}, {node.y})"
                            style="cursor: pointer;"
                            opacity={opacity}
                            onmousedown={(e) => handleNodeDragStart(e, node)}
                            onclick={() => handleNodeClick(node)}
                            onmouseenter={(e) => handleNodeMouseEnter(e, node)}
                            onmouseleave={handleNodeMouseLeave}
                            role="button"
                            tabindex="0"
                            aria-label="Nodo {node.name}"
                            onkeydown={(e) => {
                              if (e.key === 'Enter' || e.key === ' ') {
                                e.preventDefault();
                                handleNodeClick(node);
                              }
                            }}
                          >
                            <circle
                              r={radius}
                              class="node-circle {node.entity_type}"
                              stroke={isSelected ? "#ffffff" : "transparent"}
                              stroke-width="2.5"
                            />
                            <text
                              dx={radius + 5}
                              dy="3"
                              class="node-text"
                              font-weight={isSelected ? "bold" : "normal"}
                            >
                              {node.name}
                            </text>
                          </g>
                        {/if}
                      {/each}

                    </g>
                  </svg>
                {/if}

                <!-- Selected Node detail inspector overlay -->
                {#if selectedNodeId}
                  {@const selNode = graphData.nodes.find(n => n.id === selectedNodeId)}
                  {#if selNode}
                    <div class="graph-inspector glass" style="position: absolute; bottom: 12px; left: 12px; max-width: 240px; padding: 12px; border-radius: var(--radius-md); box-shadow: var(--shadow-md); z-index: 10;">
                      <h4 style="font-size: 13px; font-weight: 700; margin-bottom: 2px;">{selNode.name}</h4>
                      <span class="badge {selNode.entity_type}" style="font-size: 8px;">{selNode.entity_type}</span>
                      <p style="font-size: 11px; color: var(--text-secondary); margin-top: 6px; line-height: 1.3;">
                        Conexiones: {selNode.connections_count}
                      </p>
                      <button class="btn btn-secondary btn-sm" style="width: 100%; margin-top: 8px; font-size: 10px;" onclick={() => selectedNodeId = null}>Cerrar</button>
                    </div>
                  {/if}
                {/if}

                <!-- Legend -->
                <div class="graph-legend glass" style="position: absolute; top: 12px; right: 12px; padding: 10px; border-radius: var(--radius-md); display: flex; flex-direction: column; gap: 4px; z-index: 10; font-size: 11px; font-weight: 500;">
                  <div style="display: flex; align-items: center; gap: 6px;"><span style="display: inline-block; width: 10px; height: 10px; border-radius: 50%; background-color: #007aff;"></span> Personaje</div>
                  <div style="display: flex; align-items: center; gap: 6px;"><span style="display: inline-block; width: 10px; height: 10px; border-radius: 50%; background-color: #34c759;"></span> Ubicación</div>
                  <div style="display: flex; align-items: center; gap: 6px;"><span style="display: inline-block; width: 10px; height: 10px; border-radius: 50%; background-color: #ff9500;"></span> Objeto</div>
                  <div style="display: flex; align-items: center; gap: 6px;"><span style="display: inline-block; width: 10px; height: 10px; border-radius: 50%; background-color: #5856d6;"></span> Alianza</div>
                  <div style="display: flex; align-items: center; gap: 6px;"><span style="display: inline-block; width: 10px; height: 10px; border-radius: 50%; background-color: #ff3b30;"></span> Ley de Magia</div>
                </div>

                <!-- Hover Tooltip -->
                {#if hoveredNode}
                  <div 
                    class="graph-tooltip glass" 
                    style="position: absolute; left: {tooltipPosition.x}px; top: {tooltipPosition.y}px; padding: 6px 10px; border-radius: var(--radius-sm); pointer-events: none; z-index: 20; font-size: 11px; box-shadow: var(--shadow-sm);"
                  >
                    <span style="font-weight: 600;">{hoveredNode.name}</span>
                    <span style="opacity: 0.7;"> ({hoveredNode.entity_type})</span>
                    <br/>
                    <span style="font-size: 9px; opacity: 0.6;">{hoveredNode.connections_count} conexiones activas</span>
                  </div>
                {/if}
              {/if}
            </div>
          </div>

        {:else if activeTab === "entities"}
          <div class="split-pane animate-fade-in">
            <!-- List of Entities -->
            <div class="card glass pane-list">
              <h2>Wiki de Entidades</h2>
              {#if entities.length === 0}
                <div class="empty-state">No se han registrado entidades de worldbuilding.</div>
              {:else}
                <div class="apple-list">
                  {#each entities as ent}
                    <div class="apple-item align-top">
                      <div class="item-details">
                        <div class="item-primary">
                          {ent.name} 
                          <span class="badge {ent.entity_type}">{ent.entity_type}</span>
                        </div>
                        {#if ent.description}
                          <p class="item-body-desc">{ent.description}</p>
                        {/if}
                        <div class="item-meta">
                          Propiedades: 
                          <span class="code font-xs">Edad: {ent.properties.age ?? "N/A"} | Lealtad: {ent.properties.loyalty}</span>
                        </div>
                      </div>
                      <button class="btn btn-danger btn-sm" onclick={() => handleDeleteEntity(ent.id)}>Borrar</button>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>

            <!-- Creation Form -->
            <div class="card glass pane-form">
              <h2>Nueva Entidad</h2>
              <div class="form-group">
                <label for="ent-name">Nombre</label>
                <input id="ent-name" type="text" placeholder="Ej. Lancelot du Lac" bind:value={entName} />
              </div>
              <div class="form-group">
                <label for="ent-type">Tipo de Entidad</label>
                <select id="ent-type" bind:value={entType}>
                  <option value="character">Personaje</option>
                  <option value="location">Ubicación</option>
                  <option value="item">Objeto mágico / Item</option>
                  <option value="faction">Facciones / Alianzas</option>
                  <option value="magic">Leyes de Magia</option>
                </select>
              </div>
              <div class="form-group">
                <label for="ent-desc">Descripción Narrativa</label>
                <textarea id="ent-desc" rows="3" placeholder="Ej. Caballero más fuerte del reino" bind:value={entDesc}></textarea>
              </div>
              <div class="form-row">
                <div class="form-group half-width">
                  <label for="ent-age">Edad</label>
                  <input id="ent-age" type="number" placeholder="25" bind:value={entAge} />
                </div>
                <div class="form-group half-width">
                  <label for="ent-loyalty">Lealtad</label>
                  <select id="ent-loyalty" bind:value={entLoyalty}>
                    <option value="high">Fiel / Alta</option>
                    <option value="neutral">Neutral</option>
                    <option value="traitor">Baja / Traidor</option>
                  </select>
                </div>
              </div>
              <button class="btn btn-primary full-width margin-top-md" onclick={handleCreateEntity}>Registrar Entidad</button>
            </div>
          </div>

        {:else if activeTab === "tags"}
          <div class="split-pane animate-fade-in">
            <!-- List of Tags -->
            <div class="card glass pane-list">
              <h2>Palabras Clave Temáticas</h2>
              {#if tags.length === 0}
                <div class="empty-state">No hay palabras clave registradas.</div>
              {:else}
                <div class="apple-list">
                  {#each tags as tag}
                    <div class="apple-item">
                      <div class="item-details">
                        <div class="item-primary keyword-color">{tag.name}</div>
                        {#if tag.description}
                          <div class="item-secondary">{tag.description}</div>
                        {/if}
                      </div>
                      <button class="btn btn-danger btn-sm" onclick={() => handleDeleteTag(tag.id)}>Borrar</button>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>

            <!-- Creation Form -->
            <div class="card glass pane-form">
              <h2>Nueva Palabra Clave</h2>
              <div class="form-group">
                <label for="tag-name">Palabra Clave</label>
                <input id="tag-name" type="text" placeholder="Ej. #MagiaAntigua o Venganza" bind:value={tagName} />
              </div>
              <div class="form-group">
                <label for="tag-desc">Descripción Semántica</label>
                <input id="tag-desc" type="text" placeholder="Ej. Vinculado a arcos de venganza familiar" bind:value={tagDesc} />
              </div>
              <button class="btn btn-primary full-width margin-top-md" onclick={handleCreateTag}>Guardar Palabra</button>
            </div>
          </div>

        {:else if activeTab === "links"}
          <div class="split-pane animate-fade-in">
            <!-- List of Links -->
            <div class="card glass pane-list">
              <h2>Relaciones en el Grafo</h2>
              {#if links.length === 0}
                <div class="empty-state">No hay relaciones creadas entre tus personajes y locaciones.</div>
              {:else}
                <div class="apple-list">
                  {#each links as link}
                    <div class="apple-item">
                      <div class="item-details">
                        <div class="item-primary">
                          <span class="entity-lbl">{getEntityName(link.source_entity_id)}</span>
                          <span class="rel-arrow">➔ ({link.link_type}) ➔</span>
                          <span class="entity-lbl">{getEntityName(link.target_entity_id)}</span>
                        </div>
                        {#if link.description}
                          <div class="item-secondary">{link.description}</div>
                        {/if}
                        <div class="item-meta">Fuerza de Relación (Gravedad): {link.weight}</div>
                      </div>
                      <button class="btn btn-danger btn-sm" onclick={() => handleDeleteLink(link.id)}>Borrar</button>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>

            <!-- Creation Form -->
            <div class="card glass pane-form">
              <h2>Nueva Conexión</h2>
              <div class="form-group">
                <label for="link-source">Entidad A</label>
                <select id="link-source" bind:value={linkSource}>
                  <option value="">-- Seleccionar origen --</option>
                  {#each entities as ent}
                    <option value={ent.id}>{ent.name} ({ent.entity_type})</option>
                  {/each}
                </select>
              </div>
              <div class="form-group">
                <label for="link-target">Entidad B</label>
                <select id="link-target" bind:value={linkTarget}>
                  <option value="">-- Seleccionar destino --</option>
                  {#each entities as ent}
                    {#if ent.id !== linkSource}
                      <option value={ent.id}>{ent.name} ({ent.entity_type})</option>
                    {/if}
                  {/each}
                </select>
              </div>
              <div class="form-group">
                <label for="link-type">Naturaleza</label>
                <select id="link-type" bind:value={linkType}>
                  <option value="custom">Personalizada</option>
                  <option value="family">Familia / Sangre</option>
                  <option value="affiliation">Alianza / Gremio</option>
                  <option value="enmity">Rivalidad / Enemistad</option>
                  <option value="love">Lazos amorosos</option>
                </select>
              </div>
              <div class="form-group">
                <label for="link-desc">Detalle</label>
                <input id="link-desc" type="text" placeholder="Ej. Hermanos gemelos" bind:value={linkDesc} />
              </div>
              <div class="form-group">
                <label for="link-weight">Peso de Atracción ({linkWeight})</label>
                <input id="link-weight" type="range" min="0.1" max="2.0" step="0.1" bind:value={linkWeight} />
              </div>
              <button class="btn btn-primary full-width margin-top-md" onclick={handleCreateLink}>Enlazar Entidades</button>
            </div>
          </div>
        {:else if activeTab === "flow"}
          <div class="flow-container glass animate-fade-in">
            <!-- Toolbar -->
            <div class="flow-toolbar">
              <div class="toolbar-title">
                <h2>Editor de Flujos Narrativos</h2>
                <span class="item-secondary">Diseña bifurcaciones y lógica conversacional o de quest</span>
              </div>
              <div class="toolbar-actions">
                <button class="btn btn-primary btn-sm" onclick={() => showCreateNodeModal = true}>
                  <svg class="icon-sm" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
                  Nuevo Nodo
                </button>
                <button class="btn btn-secondary btn-sm" onclick={() => { flowPanX = 0; flowPanY = 0; }}>
                  Centrar Vista
                </button>
              </div>
            </div>

            <!-- Linking Mode Overlay Banner -->
            {#if linkingSourceNodeId}
              <div class="linking-banner">
                <span>Conectando desde <strong>{flowNodes.find(n => n.id === linkingSourceNodeId)?.title || "Nodo"}</strong>. Selecciona el nodo destino:</span>
                <button class="btn btn-secondary btn-xs" onclick={() => { linkingSourceNodeId = null; }}>Cancelar</button>
              </div>
            {/if}

            <!-- Canvas Viewport -->
            <div 
              class="flow-canvas-viewport" 
              role="presentation"
              onmousedown={onCanvasMouseDown}
            >
              <!-- Grid background -->
              <div 
                class="flow-canvas-grid" 
                style="transform: translate({flowPanX}px, {flowPanY}px) scale({flowZoom});"
              >
                <!-- SVG Connections Overlay -->
                <svg class="flow-svg-overlay">
                  <defs>
                    <marker id="arrow" viewBox="0 0 10 10" refX="28" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
                      <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--primary-light)"></path>
                    </marker>
                    <marker id="arrow-selected" viewBox="0 0 10 10" refX="28" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
                      <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--accent)"></path>
                    </marker>
                  </defs>

                  {#each flowLinks as link}
                    {@const source = flowNodes.find(n => n.id === link.source_node_id)}
                    {@const target = flowNodes.find(n => n.id === link.target_node_id)}
                    {#if source && target}
                      <!-- Source and target centers -->
                      {@const x1 = source.x + 100}
                      {@const y1 = source.y + 45}
                      {@const x2 = target.x + 100}
                      {@const y2 = target.y + 45}
                      <!-- Curve control points -->
                      {@const dx = Math.abs(x2 - x1) * 0.5}
                      {@const pathD = `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`}
                      
                      <!-- Connection Line -->
                      <path 
                        d={pathD} 
                        class="flow-link-path"
                        marker-end="url(#arrow)"
                      />
                      
                      <!-- Interactive Link Button/Label at middle -->
                      {@const mx = (x1 + x2) / 2}
                      {@const my = (y1 + y2) / 2}
                      <foreignObject x={mx - 50} y={my - 18} width="100" height="36" class="flow-link-object">
                        <div class="flow-link-badge-container">
                          {#if link.label || link.conditions}
                            <span class="flow-link-badge" title={link.conditions || link.label}>
                              {link.label || 'Condición'}
                            </span>
                          {/if}
                          <button 
                            class="flow-link-delete" 
                            title="Eliminar Conexión"
                            onclick={() => handleDeleteFlowLink(link.id)}
                          >
                            ×
                          </button>
                        </div>
                      </foreignObject>
                    {/if}
                  {/each}
                </svg>

                <!-- Flowchart Nodes -->
                {#each flowNodes as node (node.id)}
                  <!-- Node Card -->
                  <div 
                    class="flow-node-card glass"
                    class:selected={selectedFlowNode?.id === node.id}
                    class:linking-target={linkingSourceNodeId && linkingSourceNodeId !== node.id}
                    class:linking-source={linkingSourceNodeId === node.id}
                    style="left: {node.x}px; top: {node.y}px;"
                    role="presentation"
                    onmousedown={(e) => onNodeMouseDown(e, node.id)}
                  >
                    <!-- Header -->
                    <div class="node-card-header">
                      <div class="node-title" title={node.title}>{node.title}</div>
                      <div class="node-actions">
                        {#if linkingSourceNodeId}
                          {#if linkingSourceNodeId !== node.id}
                            <button 
                              class="node-action-btn connect" 
                              title="Conectar aquí"
                              onclick={() => { targetLinkNodeId = node.id; showCreateLinkModal = true; }}
                            >
                              ➔
                            </button>
                          {/if}
                        {:else}
                          <button 
                            class="node-action-btn connect" 
                            title="Crear Enlace"
                            onclick={() => startLinking(node.id)}
                          >
                            🔗
                          </button>
                        {/if}
                        <button 
                          class="node-action-btn edit" 
                          title="Editar Detalles"
                          onclick={() => startEditFlowNode(node)}
                        >
                          ✏️
                        </button>
                        <button 
                          class="node-action-btn delete" 
                          title="Eliminar Nodo"
                          onclick={() => handleDeleteFlowNode(node.id)}
                        >
                          🗑️
                        </button>
                      </div>
                    </div>

                    <!-- Body -->
                    <div class="node-card-body">
                      {#if node.content}
                        <p class="node-excerpt">{node.content}</p>
                      {:else}
                        <p class="node-excerpt empty">Sin descripción...</p>
                      {/if}
                    </div>

                    <!-- Footer / Condition -->
                    {#if node.conditions}
                      <div class="node-card-footer" title={node.conditions}>
                        <span class="cond-icon">⚙️</span>
                        <span class="cond-text">{node.conditions}</span>
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>

            <!-- Edit Node Sidebar -->
            {#if selectedFlowNode}
              <div class="editor-sidebar-panel glass animate-slide-in-right">
                <div class="sidebar-panel-header">
                  <h3>Editar Nodo</h3>
                  <button class="btn-close" onclick={() => selectedFlowNode = null}>×</button>
                </div>
                <div class="sidebar-panel-body">
                  <div class="form-group">
                    <label for="node-edit-title">Título del Nodo</label>
                    <input id="node-edit-title" type="text" bind:value={selectedFlowNodeTitle} />
                  </div>
                  <div class="form-group">
                    <label for="node-edit-content">Contenido / Sinopsis</label>
                    <textarea id="node-edit-content" rows="6" bind:value={selectedFlowNodeContent} placeholder="Escribe el texto de esta rama..."></textarea>
                  </div>
                  <div class="form-group">
                    <label for="node-edit-conditions">Condiciones Lógicas</label>
                    <input id="node-edit-conditions" type="text" bind:value={selectedFlowNodeConditions} placeholder="Ej. jugador_tiene_llave == true" />
                  </div>
                  <div class="sidebar-panel-actions">
                    <button class="btn btn-primary full-width" onclick={handleUpdateFlowNode}>Guardar Cambios</button>
                    <button class="btn btn-secondary full-width margin-top-xs" onclick={() => selectedFlowNode = null}>Cancelar</button>
                  </div>
                </div>
              </div>
            {/if}
          </div>

          <!-- Create Node Modal -->
          {#if showCreateNodeModal}
            <div class="modal-overlay animate-fade-in">
              <div class="modal-card glass animate-zoom-in">
                <h3>Nuevo Nodo de Flujo</h3>
                <div class="form-group">
                  <label for="new-node-title">Título</label>
                  <input id="new-node-title" type="text" placeholder="Ej. Decisión A o Bifurcación..." bind:value={newFlowNodeTitle} />
                </div>
                <div class="form-group">
                  <label for="new-node-content">Contenido</label>
                  <textarea id="new-node-content" rows="4" placeholder="Contenido o descripción de este tramo..." bind:value={newFlowNodeContent}></textarea>
                </div>
                <div class="form-group">
                  <label for="new-node-conds">Condiciones (Opcional)</label>
                  <input id="new-node-conds" type="text" placeholder="Ej. item_oro > 10" bind:value={newFlowNodeConditions} />
                </div>
                <div class="modal-actions">
                  <button class="btn btn-secondary" onclick={() => showCreateNodeModal = false}>Cancelar</button>
                  <button class="btn btn-primary" onclick={handleCreateFlowNode}>Crear Nodo</button>
                </div>
              </div>
            </div>
          {/if}

          <!-- Create Link Modal -->
          {#if showCreateLinkModal}
            <div class="modal-overlay animate-fade-in">
              <div class="modal-card glass animate-zoom-in">
                <h3>Configurar Conexión</h3>
                <p>Enlazando <strong>{flowNodes.find(n => n.id === linkingSourceNodeId)?.title || "Nodo A"}</strong> con <strong>{flowNodes.find(n => n.id === targetLinkNodeId)?.title || "Nodo B"}</strong>.</p>
                <div class="form-group">
                  <label for="link-label">Etiqueta del Enlace (Ej. Decisión o Acción)</label>
                  <input id="link-label" type="text" placeholder="Ej. Escapar por el norte" bind:value={newFlowLinkLabel} />
                </div>
                <div class="form-group">
                  <label for="link-conditions">Condiciones del Enlace</label>
                  <input id="link-conditions" type="text" placeholder="Ej. tiene_mapa" bind:value={newFlowLinkConditions} />
                </div>
                <div class="modal-actions">
                  <button class="btn btn-secondary" onclick={() => { showCreateLinkModal = false; linkingSourceNodeId = null; }}>Cancelar</button>
                  <button class="btn btn-primary" onclick={handleCreateFlowLink}>Crear Conexión</button>
                </div>
              </div>
            </div>
          {/if}

        {:else if activeTab === "timeline"}
          <div class="timeline-container split-pane animate-fade-in">
            <!-- Timeline Main list -->
            <div class="card glass pane-list timeline-main-list">
              <div class="timeline-header-bar">
                <h2>Línea Temporal Histórica</h2>
                <div class="timeline-controls">
                  <input 
                    type="text" 
                    placeholder="Buscar eventos..." 
                    class="timeline-search"
                    bind:value={timelineSearchQuery}
                  />
                  <button 
                    class="btn btn-secondary btn-sm"
                    onclick={() => timelineSortAsc = !timelineSortAsc}
                    title={timelineSortAsc ? "Orden Ascendente" : "Orden Descendente"}
                  >
                    {timelineSortAsc ? "↑ Cronológico" : "↓ Inverso"}
                  </button>
                  <button 
                    class="btn btn-primary btn-sm"
                    onclick={() => showCreateTimelineEvent = true}
                  >
                    + Nuevo Evento
                  </button>
                </div>
              </div>

              {#if filteredTimelineEvents.length === 0}
                <div class="empty-state">No se encontraron eventos en la línea de tiempo.</div>
              {:else}
                <div class="timeline-scroll-area">
                  <div class="timeline-axis"></div>
                  <div class="timeline-events-list">
                    {#each filteredTimelineEvents as ev}
                      <div class="timeline-event-card glass animate-fade-in">
                        <!-- Node indicator dot on timeline axis -->
                        <div class="timeline-dot"></div>
                        
                        <div class="event-card-header">
                          <div class="event-date-badge">{ev.event_date}</div>
                          <h3 class="event-title">{ev.title}</h3>
                        </div>
                        
                        <div class="event-card-body">
                          {#if ev.description}
                            <p class="event-desc">{ev.description}</p>
                          {/if}
                          
                          {#if ev.associated_entity_id}
                            {@const assoc = entities.find(e => e.id === ev.associated_entity_id)}
                            {#if assoc}
                              <div class="event-entity-badge {assoc.entity_type}">
                                <span class="badge-dot"></span>
                                {assoc.name} ({assoc.entity_type})
                              </div>
                            {/if}
                          {/if}
                        </div>

                        <div class="event-card-actions">
                          <button class="btn btn-secondary btn-xs" onclick={() => startEditTimelineEvent(ev)}>✏️ Editar</button>
                          <button class="btn btn-danger btn-xs" onclick={() => handleDeleteTimelineEvent(ev.id)}>🗑️ Borrar</button>
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>

            <!-- Edit Event Sidebar Panel -->
            {#if selectedTimelineEvent}
              <div class="editor-sidebar-panel glass animate-slide-in-right">
                <div class="sidebar-panel-header">
                  <h3>Editar Evento</h3>
                  <button class="btn-close" onclick={() => selectedTimelineEvent = null}>×</button>
                </div>
                <div class="sidebar-panel-body">
                  <div class="form-group">
                    <label for="event-edit-title">Título del Evento</label>
                    <input id="event-edit-title" type="text" bind:value={timelineEventTitle} />
                  </div>
                  <div class="form-group">
                    <label for="event-edit-date">Fecha / Periodo</label>
                    <input id="event-edit-date" type="text" placeholder="Ej. A.C. 500, Año 14, 2026-06-08" bind:value={timelineEventDate} />
                  </div>
                  <div class="form-group">
                    <label for="event-edit-desc">Descripción</label>
                    <textarea id="event-edit-desc" rows="5" bind:value={timelineEventDescription}></textarea>
                  </div>
                  <div class="form-group">
                    <label for="event-edit-entity">Entidad Asociada</label>
                    <select id="event-edit-entity" bind:value={timelineEventEntityId}>
                      <option value="">-- Ninguna --</option>
                      {#each entities as ent}
                        <option value={ent.id}>{ent.name} ({ent.entity_type})</option>
                      {/each}
                    </select>
                  </div>
                  <div class="sidebar-panel-actions">
                    <button class="btn btn-primary full-width" onclick={handleUpdateTimelineEvent}>Guardar Evento</button>
                    <button class="btn btn-secondary full-width margin-top-xs" onclick={() => selectedTimelineEvent = null}>Cancelar</button>
                  </div>
                </div>
              </div>
            {/if}
          </div>

          <!-- Create Event Modal -->
          {#if showCreateTimelineEvent}
            <div class="modal-overlay animate-fade-in">
              <div class="modal-card glass animate-zoom-in">
                <h3>Nuevo Evento de Línea de Tiempo</h3>
                <div class="form-group">
                  <label for="new-event-title">Título del Evento</label>
                  <input id="new-event-title" type="text" placeholder="Ej. Fundación de Valoria, Nacimiento de Arthur..." bind:value={newTimelineEventTitle} />
                </div>
                <div class="form-group">
                  <label for="new-event-date">Fecha / Año / Marca Temporal</label>
                  <input id="new-event-date" type="text" placeholder="Ej. 1200 d.C., Año -50, Día 3" bind:value={newTimelineEventDate} />
                </div>
                <div class="form-group">
                  <label for="new-event-desc">Descripción Histórica</label>
                  <textarea id="new-event-desc" rows="4" placeholder="Describe qué ocurrió en este momento..." bind:value={newTimelineEventDescription}></textarea>
                </div>
                <div class="form-group">
                  <label for="new-event-entity">Entidad Relacionada (Ej. Protagonista o Ciudad)</label>
                  <select id="new-event-entity" bind:value={newTimelineEventEntityId}>
                    <option value="">-- Ninguna --</option>
                    {#each entities as ent}
                      <option value={ent.id}>{ent.name} ({ent.entity_type})</option>
                    {/each}
                  </select>
                </div>
                <div class="modal-actions">
                  <button class="btn btn-secondary" onclick={() => showCreateTimelineEvent = false}>Cancelar</button>
                  <button class="btn btn-primary" onclick={handleCreateTimelineEvent}>Crear Evento</button>
                </div>
              </div>
            </div>
          {/if}
        {/if}
      </main>

    </div>
  {/if}
</div>

<style>
  /* Flowchart Narrative Editor */
  .flow-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    position: relative;
  }

  .flow-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-3) var(--space-4);
    background: rgba(30, 30, 40, 0.4);
    backdrop-filter: blur(10px);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .flow-toolbar h2 {
    margin: 0 0 2px 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .toolbar-actions {
    display: flex;
    gap: 8px;
  }

  .linking-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-2) var(--space-4);
    background: rgba(255, 149, 0, 0.2);
    border: 1px solid rgba(255, 149, 0, 0.4);
    color: var(--accent);
    font-size: 0.85rem;
    border-bottom: 1px solid var(--accent);
    z-index: 10;
  }

  .linking-banner strong {
    color: var(--text-light);
  }

  .flow-canvas-viewport {
    flex: 1;
    position: relative;
    overflow: hidden;
    cursor: grab;
    background-color: #121218;
    background-image: 
      radial-gradient(rgba(255, 255, 255, 0.03) 1px, transparent 0),
      radial-gradient(rgba(255, 255, 255, 0.03) 1px, transparent 0);
    background-size: 20px 20px;
    background-position: 0 0, 10px 10px;
  }

  .flow-canvas-viewport:active {
    cursor: grabbing;
  }

  .flow-canvas-grid {
    position: absolute;
    width: 10000px;
    height: 10000px;
    transform-origin: 0 0;
  }

  .flow-svg-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 1;
  }

  .flow-link-path {
    fill: none;
    stroke: rgba(255, 255, 255, 0.15);
    stroke-width: 2.5px;
    transition: stroke 0.2s;
    pointer-events: stroke;
  }

  .flow-link-path:hover {
    stroke: var(--accent);
    stroke-width: 3.5px;
  }

  .flow-link-object {
    pointer-events: auto;
  }

  .flow-link-badge-container {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 4px;
  }

  .flow-link-badge {
    background: rgba(30, 30, 40, 0.95);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: var(--text-muted);
    font-size: 0.72rem;
    padding: 2px 6px;
    border-radius: 4px;
    max-width: 80px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .flow-link-delete {
    background: rgba(255, 59, 48, 0.2);
    border: 1px solid rgba(255, 59, 48, 0.4);
    color: #ff453a;
    font-size: 0.75rem;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
  }

  .flow-link-delete:hover {
    background: rgba(255, 59, 48, 0.9);
    color: white;
  }

  .flow-node-card {
    position: absolute;
    width: 200px;
    border-radius: 10px;
    background: rgba(28, 28, 36, 0.85);
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.25);
    z-index: 2;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    transition: transform 0.15s, border-color 0.2s, box-shadow 0.2s;
    user-select: none;
  }

  .flow-node-card:hover {
    border-color: rgba(255, 255, 255, 0.2);
    box-shadow: 0 8px 30px rgba(0, 0, 0, 0.4);
  }

  .flow-node-card.selected {
    border-color: var(--accent);
    box-shadow: 0 0 12px rgba(255, 149, 0, 0.3), 0 4px 20px rgba(0, 0, 0, 0.3);
  }

  .flow-node-card.linking-target {
    border-color: rgba(52, 199, 89, 0.4);
    cursor: cell;
  }

  .flow-node-card.linking-target:hover {
    border-color: rgba(52, 199, 89, 0.9);
    transform: scale(1.02);
  }

  .flow-node-card.linking-source {
    border-color: var(--accent);
    opacity: 0.8;
  }

  .node-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-2) var(--space-3);
    background: rgba(255, 255, 255, 0.03);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .node-title {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-light);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100px;
  }

  .node-actions {
    display: flex;
    gap: 4px;
  }

  .node-action-btn {
    background: transparent;
    border: none;
    font-size: 0.75rem;
    cursor: pointer;
    padding: 2px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    transition: all 0.2s;
  }

  .node-action-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-light);
  }

  .node-action-btn.delete:hover {
    background: rgba(255, 59, 48, 0.15);
    color: #ff453a;
  }

  .node-action-btn.connect:hover {
    background: rgba(52, 199, 89, 0.15);
    color: #30d158;
  }

  .node-card-body {
    padding: var(--space-2) var(--space-3);
    flex: 1;
    display: flex;
    align-items: center;
    min-height: 45px;
  }

  .node-excerpt {
    font-size: 0.75rem;
    color: var(--text-muted);
    line-height: 1.35;
    margin: 0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .node-excerpt.empty {
    font-style: italic;
    opacity: 0.5;
  }

  .node-card-footer {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px var(--space-3);
    background: rgba(255, 255, 255, 0.015);
    border-top: 1px solid rgba(255, 255, 255, 0.04);
    font-size: 0.65rem;
    color: var(--accent);
  }

  .cond-text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 160px;
  }

  /* Edit Node Sidebar Panel */
  .editor-sidebar-panel {
    position: absolute;
    right: 0;
    top: 0;
    width: 320px;
    height: 100%;
    background: rgba(24, 24, 32, 0.9);
    backdrop-filter: blur(20px);
    border-left: 1px solid rgba(255, 255, 255, 0.08);
    z-index: 100;
    display: flex;
    flex-direction: column;
    box-shadow: -4px 0 24px rgba(0, 0, 0, 0.4);
  }

  .sidebar-panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .sidebar-panel-header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .btn-close {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0 4px;
  }

  .btn-close:hover {
    color: var(--text-light);
  }

  .sidebar-panel-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-4);
  }

  .sidebar-panel-actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: var(--space-4);
  }

  /* Chronological Timeline */
  .timeline-container {
    display: flex;
    height: 100%;
    position: relative;
  }

  .timeline-main-list {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .timeline-header-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: var(--space-3);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    margin-bottom: var(--space-4);
    flex-wrap: wrap;
    gap: 12px;
  }

  .timeline-controls {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .timeline-search {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 6px;
    padding: 6px 12px;
    color: var(--text-light);
    font-size: 0.85rem;
    outline: none;
    transition: all 0.2s;
  }

  .timeline-search:focus {
    background: rgba(255, 255, 255, 0.08);
    border-color: var(--accent);
  }

  .timeline-scroll-area {
    flex: 1;
    overflow-y: auto;
    position: relative;
    padding: var(--space-4) var(--space-2);
  }

  .timeline-axis {
    position: absolute;
    left: 20px;
    top: 0;
    bottom: 0;
    width: 2px;
    background: linear-gradient(to bottom, transparent, rgba(255, 255, 255, 0.15) 10%, rgba(255, 255, 255, 0.15) 90%, transparent);
    z-index: 1;
  }

  .timeline-events-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    position: relative;
    z-index: 2;
    padding-left: 40px;
  }

  .timeline-event-card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    padding: var(--space-3) var(--space-4);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
    position: relative;
    transition: all 0.2s;
  }

  .timeline-event-card:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.12);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.15);
  }

  .timeline-dot {
    position: absolute;
    left: -26px;
    top: 22px;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--accent);
    border: 2px solid #16161e;
    box-shadow: 0 0 8px var(--accent);
    z-index: 3;
  }

  .event-card-header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-bottom: var(--space-2);
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    padding-bottom: 4px;
  }

  .event-date-badge {
    background: rgba(255, 149, 0, 0.15);
    border: 1px solid rgba(255, 149, 0, 0.3);
    color: var(--accent);
    font-size: 0.75rem;
    font-weight: 700;
    padding: 2px 8px;
    border-radius: 4px;
    white-space: nowrap;
  }

  .event-title {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-light);
  }

  .event-card-body {
    font-size: 0.85rem;
    color: var(--text-muted);
    line-height: 1.45;
    margin-bottom: var(--space-2);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .event-desc {
    margin: 0;
    white-space: pre-wrap;
  }

  .event-entity-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 0.75rem;
    padding: 2px 8px;
    border-radius: 12px;
    width: fit-content;
  }

  .event-entity-badge.character {
    background: rgba(0, 122, 255, 0.15);
    border: 1px solid rgba(0, 122, 255, 0.3);
    color: #007aff;
  }
  .event-entity-badge.location {
    background: rgba(52, 199, 89, 0.15);
    border: 1px solid rgba(52, 199, 89, 0.3);
    color: #34c759;
  }
  .event-entity-badge.item {
    background: rgba(255, 149, 0, 0.15);
    border: 1px solid rgba(255, 149, 0, 0.3);
    color: #ff9500;
  }
  .event-entity-badge.faction {
    background: rgba(88, 86, 214, 0.15);
    border: 1px solid rgba(88, 86, 214, 0.3);
    color: #5856d6;
  }
  .event-entity-badge.magic {
    background: rgba(255, 59, 48, 0.15);
    border: 1px solid rgba(255, 59, 48, 0.3);
    color: #ff3b30;
  }

  .badge-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }

  .event-card-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    border-top: 1px solid rgba(255, 255, 255, 0.03);
    padding-top: var(--space-2);
  }

  .btn-xs {
    padding: 2px 8px;
    font-size: 0.75rem;
  }
  /* 2D Graph Visual Elements */
  .graph-svg {
    user-select: none;
    -webkit-user-select: none;
  }

  .node-circle {
    stroke-width: 2px;
    transition: r 0.2s var(--transition-fast), stroke 0.2s;
  }

  .node-circle.character { fill: #007aff; }
  .node-circle.location { fill: #34c759; }
  .node-circle.item { fill: #ff9500; }
  .node-circle.faction { fill: #5856d6; }
  .node-circle.magic { fill: #ff3b30; }

  .node-circle:hover {
    stroke: #ffffff;
    stroke-width: 3px;
  }

  .node-text {
    font-size: 10px;
    fill: var(--text-primary);
    pointer-events: none;
    font-family: var(--font-sans);
  }

  /* Performance overrides for GPU-less devices */
  :global(.low-perf-mode) {
    --glass-blur: 0px !important;
    --transition-smooth: 0s !important;
    --transition-fast: 0s !important;
    --transition-spring: 0s !important;
  }

  :global(.low-perf-mode) .glass {
    backdrop-filter: none !important;
    -webkit-backdrop-filter: none !important;
    background-color: var(--bg-window) !important;
    box-shadow: none !important;
  }

  :global(.low-perf-mode) * {
    transition: none !important;
    animation: none !important;
  }

  /* Editor Workspace layout */
  .editor-workspace {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
  }

  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-2) var(--space-4);
    border-bottom: 1px solid var(--border-primary);
    background-color: var(--bg-window);
    height: 48px;
    flex-shrink: 0;
  }

  .header-left, .header-right {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .editor-doc-title {
    font-weight: 600;
    font-size: 14px;
    margin-left: var(--space-3);
  }

  .header-center {
    display: flex;
    background-color: var(--bg-system);
    padding: 2px;
    border-radius: var(--radius-md);
  }

  .tab-btn {
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    padding: 4px 12px;
    font-size: 12.5px;
    font-weight: 500;
    color: var(--text-secondary);
    transition: all var(--transition-fast);
  }

  .tab-btn:hover {
    color: var(--text-primary);
  }

  .tab-btn.active {
    background-color: var(--bg-window);
    color: var(--text-primary);
    box-shadow: var(--shadow-sm);
  }

  .save-indicator {
    font-size: 11px;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .save-indicator.guardando {
    color: var(--accent);
  }
  
  .save-indicator.sin_guardar {
    color: var(--warning);
  }

  .spinner {
    width: 10px;
    height: 10px;
    border: 1.5px solid var(--accent);
    border-top-color: transparent;
    border-radius: var(--radius-full);
    animation: spin 0.8s linear infinite;
    display: inline-block;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .editor-container {
    display: flex;
    flex-grow: 1;
    overflow: hidden;
    height: calc(100% - 48px);
  }

  .editor-canvas-wrapper {
    flex-grow: 1;
    height: 100%;
    overflow-y: auto;
    position: relative;
    background-color: var(--bg-window);
  }

  .textarea-relative-container {
    position: relative;
    height: 100%;
    width: 100%;
  }

  .editor-textarea {
    width: 100%;
    height: 100%;
    border: none !important;
    background: transparent !important;
    resize: none;
    padding: var(--space-5);
    outline: none !important;
    box-shadow: none !important;
    font-size: 15px;
    line-height: 1.6;
    color: var(--text-primary);
    font-family: var(--font-sans);
  }

  .editor-textarea.screenplay {
    font-family: var(--font-mono) !important;
    font-size: 14px;
    padding-left: 20%;
    padding-right: 20%;
  }

  /* Live Analysis Sidebar */
  .analysis-sidebar {
    width: 320px;
    border-left: 1px solid var(--border-primary);
    height: 100%;
    overflow-y: auto;
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    flex-shrink: 0;
  }

  .analysis-section {
    border-bottom: 1px solid var(--border-secondary);
    padding-bottom: var(--space-3);
  }

  .analysis-section:last-child {
    border-bottom: none;
  }

  .analysis-section h3 {
    font-size: 14px;
    font-weight: 600;
    margin-bottom: var(--space-3);
  }

  .analysis-section h4 {
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    color: var(--text-secondary);
    letter-spacing: 0.05em;
    margin-bottom: var(--space-2);
  }

  .stats-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-2);
  }

  .stat-card {
    background-color: var(--bg-system);
    padding: var(--space-2);
    border-radius: var(--radius-md);
    text-align: center;
    border: 1px solid var(--border-secondary);
  }

  .stat-num {
    display: block;
    font-size: 20px;
    font-weight: 700;
    color: var(--accent);
  }

  .stat-lbl {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .section-empty {
    font-size: 12px;
    color: var(--text-tertiary);
    font-style: italic;
    padding: var(--space-1) 0;
  }

  .pill-cloud {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .entity-pill {
    font-size: 11px;
    padding: 3px 8px;
    border-radius: var(--radius-full);
    font-weight: 500;
  }

  .entity-pill.character { background: rgba(0, 122, 255, 0.12); color: var(--accent); }
  .entity-pill.location { background: rgba(52, 199, 89, 0.12); color: var(--success); }
  .entity-pill.item { background: rgba(255, 149, 0, 0.12); color: var(--warning); }
  .entity-pill.faction { background: rgba(88, 86, 214, 0.12); color: var(--accent-secondary); }
  .entity-pill.magic { background: rgba(255, 59, 48, 0.12); color: var(--danger); }

  .tag-pill {
    font-size: 11px;
    padding: 3px 8px;
    border-radius: var(--radius-full);
    background-color: var(--border-primary);
    color: var(--text-primary);
    font-weight: 500;
  }

  .relation-predictions {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .relation-prediction-item {
    display: flex;
    align-items: center;
    background-color: var(--bg-system);
    padding: 6px var(--space-3);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-secondary);
    font-size: 12px;
  }

  .rel-node {
    font-weight: 600;
    color: var(--text-primary);
  }

  .rel-connector {
    margin: 0 var(--space-1);
    opacity: 0.7;
  }

  .rel-status {
    margin-left: auto;
    font-size: 9px;
  }

  /* Autocomplete Dropdown styling */
  .autocomplete-dropdown {
    position: absolute;
    z-index: 200;
    width: 200px;
    max-height: 180px;
    overflow-y: auto;
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    border: 1px solid var(--border-primary);
    display: flex;
    flex-direction: column;
    padding: 4px;
    background: var(--bg-window);
  }

  .autocomplete-item {
    display: flex;
    align-items: center;
    padding: 6px var(--space-2);
    border-radius: var(--radius-sm);
    font-size: 12.5px;
    color: var(--text-primary);
    background: transparent;
    text-align: left;
    width: 100%;
  }

  .autocomplete-item:hover, .autocomplete-item.selected {
    background-color: var(--accent);
    color: #ffffff;
  }

  .ac-symbol {
    font-weight: bold;
    margin-right: 2px;
    opacity: 0.8;
  }

  .ac-name {
    font-weight: 500;
  }

  .ac-type {
    margin-left: auto;
    font-size: 10px;
    opacity: 0.6;
    text-transform: uppercase;
    font-weight: bold;
  }

  .autocomplete-item:hover .ac-type, .autocomplete-item.selected .ac-type {
    color: #ffffff;
    opacity: 0.9;
  }

  /* Unregistered Suggestions */
  .suggestions-section {
    background-color: rgba(0, 122, 255, 0.03);
    border: 1px dashed var(--accent);
    border-radius: var(--radius-md);
    padding: var(--space-3);
    margin-top: var(--space-3);
  }

  .section-subtitle {
    font-size: 11px;
    color: var(--text-secondary);
    line-height: 1.4;
    margin-bottom: var(--space-2);
  }

  .suggestion-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-1) 0;
    border-bottom: 1px solid var(--border-secondary);
  }

  .suggestion-item:last-child {
    border-bottom: none;
  }

  .suggestion-name {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .suggestion-actions {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .select-xs {
    font-size: 11px !important;
    padding: 2px 4px !important;
    width: auto !important;
    height: 22px !important;
    border-radius: var(--radius-sm) !important;
  }

  .input-xs {
    font-size: 11px !important;
    padding: 2px 6px !important;
    width: 90px !important;
    height: 22px !important;
    border-radius: var(--radius-sm) !important;
  }

  .btn-xs {
    padding: 2px 6px !important;
    font-size: 11px !important;
    height: 22px !important;
    border-radius: var(--radius-sm) !important;
  }

  /* Previews and Paper Simulations */
  .preview-container {
    padding: var(--space-6);
    background-color: var(--bg-system);
    min-height: 100%;
    display: flex;
    justify-content: center;
  }

  .prose-paper, .screenplay-paper {
    background-color: #ffffff;
    color: #1d1d1f;
    box-shadow: var(--shadow-lg);
    border: 1px solid var(--border-primary);
    padding: 40px 50px;
    max-width: 800px;
    width: 100%;
    min-height: 842px; /* A4 Ratio */
    border-radius: var(--radius-sm);
  }

  .prose-paper {
    font-family: var(--font-sans);
    line-height: 1.7;
    font-size: 16px;
  }

  .prose-paper :global(h1) { font-size: 26px; font-weight: 700; margin-bottom: var(--space-4); margin-top: var(--space-4); }
  .prose-paper :global(h2) { font-size: 20px; font-weight: 600; margin-bottom: var(--space-3); margin-top: var(--space-4); }
  .prose-paper :global(h3) { font-size: 16px; font-weight: 600; margin-bottom: var(--space-2); margin-top: var(--space-3); }
  .prose-paper :global(p) { margin-bottom: var(--space-4); text-align: justify; }

  /* Screenplay Fountain Rendering Styles */
  .screenplay-paper {
    font-family: "Courier New", Courier, monospace !important;
    font-size: 12pt !important;
    line-height: 1.2 !important;
    padding: 1in !important; /* Hollywood standard 1 inch margins */
    color: #000000 !important;
  }

  .screenplay-paper :global(.fountain-scene-heading) {
    text-transform: uppercase;
    font-weight: bold;
    margin-top: 1.5em;
    margin-bottom: 1em;
  }

  .screenplay-paper :global(.fountain-action) {
    margin-bottom: 1em;
    text-align: left;
  }

  .screenplay-paper :global(.fountain-character) {
    text-align: center;
    margin-top: 1.5em;
    margin-bottom: 0.2em;
    width: 60%;
    margin-left: 20%;
    font-weight: bold;
  }

  .screenplay-paper :global(.fountain-parenthetical) {
    text-align: left;
    margin-bottom: 0.2em;
    width: 40%;
    margin-left: 30%;
  }

  .screenplay-paper :global(.fountain-dialogue) {
    text-align: left;
    margin-bottom: 1em;
    width: 60%;
    margin-left: 20%;
  }

  .screenplay-paper :global(.fountain-transition) {
    text-align: right;
    margin-top: 1em;
    margin-bottom: 1.5em;
    text-transform: uppercase;
  }

  .screenplay-paper :global(.fountain-centered) {
    text-align: center;
    margin: 1em 0;
  }

  .screenplay-paper :global(.fountain-mention), .prose-paper :global(.fountain-mention) {
    color: var(--accent-secondary);
    font-weight: bold;
    background-color: rgba(88, 86, 214, 0.08);
    padding: 0 4px;
    border-radius: 3px;
  }

  .screenplay-paper :global(.fountain-tag), .prose-paper :global(.fountain-tag) {
    color: var(--accent);
    font-weight: bold;
    background-color: rgba(0, 122, 255, 0.08);
    padding: 0 4px;
    border-radius: 3px;
  }

  /* Dark mode specific preview adjustments */
  :global(.dark-mode) .prose-paper, :global(.dark-mode) .screenplay-paper {
    background-color: #1c1c1e !important;
    color: #f5f5f7 !important;
    border-color: var(--border-primary) !important;
  }

  /* Empty state */
  .empty-preview {
    color: var(--text-secondary);
    font-style: italic;
    text-align: center;
    margin-top: 40px;
  }

  /* Zen Mode rules */
  .editor-workspace.zen .editor-header {
    background-color: transparent;
    border-bottom: none;
    opacity: 0.4;
    transition: opacity var(--transition-smooth);
  }

  .editor-workspace.zen .editor-header:hover {
    opacity: 1;
  }

  .editor-workspace.zen .editor-container {
    height: 100%;
  }

  .editor-workspace.zen .editor-canvas-wrapper {
    background-color: var(--bg-system);
  }

  .editor-workspace.zen .editor-textarea {
    max-width: 800px;
    margin: 0 auto;
    padding-top: var(--space-8);
  }

  /* macOS Application Frame Layout */
  .window-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background-color: var(--bg-system);
    transition: background-color var(--transition-smooth);
  }

  .window-header {
    height: 52px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-4);
    border-bottom: 1px solid var(--border-primary);
    z-index: 50;
    box-shadow: var(--shadow-sm);
  }

  /* macOS Traffic Light Window Controls */
  .window-controls {
    display: flex;
    gap: 8px;
    width: 80px;
  }

  .dot {
    width: 12px;
    height: 12px;
    border-radius: var(--radius-full);
    display: inline-block;
  }

  .dot.red { background-color: #ff5f56; }
  .dot.yellow { background-color: #ffbd2e; }
  .dot.green { background-color: #27c93f; }

  .title-bar {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 13px;
    font-weight: 600;
  }

  .app-title {
    color: var(--accent);
    letter-spacing: 0.05em;
    text-transform: uppercase;
    font-size: 11px;
  }

  .project-title {
    color: var(--text-primary);
  }

  .separator {
    color: var(--text-tertiary);
  }

  .window-actions {
    display: flex;
    justify-content: flex-end;
    width: 80px;
  }

  .theme-toggle {
    background: transparent;
    padding: 6px;
    border-radius: var(--radius-md);
    color: var(--text-secondary);
  }

  .theme-toggle:hover {
    background-color: var(--border-primary);
    color: var(--text-primary);
  }

  .theme-icon {
    width: 18px;
    height: 18px;
  }

  /* Initial landing workspace loader */
  .project-loader {
    display: flex;
    justify-content: center;
    align-items: center;
    flex-grow: 1;
    background-color: var(--bg-system);
    padding: var(--space-4);
  }

  .loader-card {
    padding: var(--space-6);
    border-radius: var(--radius-xl);
    max-width: 480px;
    width: 100%;
    text-align: center;
    box-shadow: var(--shadow-lg);
    animation: scaleUp var(--transition-smooth);
  }

  @keyframes scaleUp {
    from { transform: scale(0.95); opacity: 0; }
    to { transform: scale(1); opacity: 1; }
  }

  .branding-logo {
    display: inline-flex;
    padding: var(--space-3);
    background: rgba(0, 122, 255, 0.1);
    border-radius: var(--radius-lg);
    color: var(--accent);
    margin-bottom: var(--space-4);
  }

  .brand-svg {
    width: 42px;
    height: 42px;
  }

  .loader-card h2 {
    font-size: 20px;
    font-weight: 600;
    margin-bottom: var(--space-2);
  }

  .loader-card p {
    color: var(--text-secondary);
    font-size: 14px;
    line-height: 1.5;
    margin-bottom: var(--space-5);
  }

  .loader-input-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  /* Project Sidebar & Layout */
  .workspace-layout {
    display: grid;
    grid-template-columns: 240px 1fr;
    flex-grow: 1;
    height: calc(100vh - 52px);
    overflow: hidden;
  }

  .sidebar {
    background-color: var(--bg-sidebar);
    border-right: 1px solid var(--border-primary);
    padding: var(--space-4) var(--space-3);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    height: 100%;
  }

  .nav-menu {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .nav-section-title {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    color: var(--text-secondary);
    letter-spacing: 0.05em;
    margin-top: var(--space-4);
    margin-bottom: var(--space-2);
    padding-left: var(--space-2);
  }

  .nav-btn {
    display: flex;
    width: 100%;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    font-size: 13.5px;
    font-weight: 500;
    text-align: left;
    background: transparent;
    transition: all var(--transition-fast);
  }

  .nav-btn:hover {
    background-color: var(--border-secondary);
    color: var(--text-primary);
  }

  .nav-btn.active {
    background-color: var(--accent);
    color: #ffffff;
  }

  .nav-btn .icon {
    width: 16px;
    height: 16px;
  }

  .sidebar-footer {
    border-top: 1px solid var(--border-primary);
    padding-top: var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .path-display {
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    padding-left: var(--space-1);
  }

  .path-display .label {
    color: var(--text-secondary);
    margin-right: 4px;
  }

  .path-display .value {
    color: var(--text-primary);
    font-weight: 600;
  }

  .panel-content {
    padding: var(--space-5);
    background-color: var(--bg-system);
    overflow-y: auto;
    height: 100%;
  }

  /* General elements: Cards & Lists */
  .card {
    background-color: var(--bg-window);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-primary);
    padding: var(--space-5);
    box-shadow: var(--shadow-md);
    transition: background-color var(--transition-smooth);
  }

  .card h2 {
    font-size: 18px;
    font-weight: 600;
    margin-bottom: var(--space-4);
    letter-spacing: -0.015em;
  }

  .apple-table {
    display: flex;
    flex-direction: column;
  }

  .table-row {
    display: grid;
    grid-template-columns: 200px 1fr;
    padding: var(--space-3) 0;
    border-bottom: 1px solid var(--border-secondary);
  }

  .table-row:last-child {
    border-bottom: none;
  }

  .col-lbl {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .col-val {
    color: var(--text-primary);
  }

  .col-val.code {
    font-family: var(--font-mono);
    background-color: var(--bg-system);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: 12px;
  }

  /* Split Pane Layout for lists + forms */
  .split-pane {
    display: grid;
    grid-template-columns: 1.25fr 0.75fr;
    gap: var(--space-5);
    align-items: start;
  }

  .pane-list {
    min-height: 400px;
  }

  .pane-form {
    position: sticky;
    top: 0;
  }

  /* Form Elements extensions */
  .form-row {
    display: flex;
    gap: var(--space-3);
  }

  .half-width {
    flex: 1;
  }

  .full-width {
    width: 100%;
  }

  .margin-top-md {
    margin-top: var(--space-4);
  }

  /* Buttons */
  .btn {
    border-radius: var(--radius-md);
    padding: var(--space-2) var(--space-4);
    font-size: 13.5px;
    font-weight: 500;
    letter-spacing: -0.01em;
  }

  .btn-primary {
    background-color: var(--accent);
    color: #ffffff;
  }

  .btn-primary:hover {
    background-color: var(--accent-hover);
  }

  .btn-secondary {
    background-color: var(--border-primary);
    color: var(--text-primary);
  }

  .btn-secondary:hover {
    background-color: var(--border-secondary);
  }

  .btn-danger {
    background-color: rgba(255, 59, 48, 0.1);
    color: var(--danger);
  }

  .btn-danger:hover {
    background-color: var(--danger);
    color: #ffffff;
  }

  .btn-sm {
    padding: var(--space-1) var(--space-3);
    font-size: 11.5px;
    border-radius: var(--radius-sm);
  }

  /* Apple List Elements */
  .apple-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .apple-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background-color: var(--bg-system);
    border: 1px solid var(--border-secondary);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    transition: transform var(--transition-fast);
  }

  .apple-item:hover {
    transform: translateY(-1px);
    border-color: var(--border-primary);
  }

  .apple-item.align-top {
    align-items: flex-start;
  }

  .item-details {
    flex-grow: 1;
    padding-right: var(--space-3);
  }

  .item-primary {
    font-weight: 600;
    font-size: 14.5px;
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .item-secondary {
    font-size: 12px;
    color: var(--text-secondary);
    margin-top: 2px;
  }

  .item-meta {
    font-size: 11px;
    color: var(--text-secondary);
    margin-top: 4px;
    border-top: 1px solid rgba(0,0,0,0.02);
    padding-top: 4px;
  }

  .item-body-desc {
    font-size: 13px;
    color: var(--text-secondary);
    margin-top: 4px;
    line-height: 1.4;
  }

  .doc-icon {
    font-size: 14px;
    margin-right: var(--space-1);
  }

  .keyword-color {
    color: var(--accent-secondary);
  }

  .entity-lbl {
    color: var(--text-primary);
  }

  .rel-arrow {
    color: var(--accent);
    font-size: 11px;
    font-weight: bold;
    margin: 0 4px;
  }

  /* Badges */
  .badge {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    letter-spacing: 0.02em;
  }

  .badge.character { background: rgba(0, 122, 255, 0.1); color: var(--accent); }
  .badge.location { background: rgba(52, 199, 89, 0.1); color: var(--success); }
  .badge.item { background: rgba(255, 149, 0, 0.1); color: var(--warning); }
  .badge.faction { background: rgba(88, 86, 214, 0.1); color: var(--accent-secondary); }
  .badge.magic { background: rgba(255, 59, 48, 0.1); color: var(--danger); }

  /* Empty States */
  .empty-state {
    color: var(--text-secondary);
    text-align: center;
    font-style: italic;
    padding: var(--space-6) 0;
    font-size: 13.5px;
  }

  /* Alerts */
  .alert {
    position: fixed;
    bottom: 24px;
    right: 24px;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-lg);
    font-size: 13px;
    font-weight: 500;
    z-index: 100;
    box-shadow: var(--shadow-lg);
    animation: alertSlide var(--transition-spring);
  }

  @keyframes alertSlide {
    from { transform: translateY(120px) scale(0.95); opacity: 0; }
    to { transform: translateY(0) scale(1); opacity: 1; }
  }

  .alert-error {
    background-color: var(--danger);
    color: #ffffff;
  }

  .alert-success {
    background-color: var(--success);
    color: #ffffff;
  }

  /* Fade-in Animation */
  .animate-fade-in {
    animation: fadeIn var(--transition-smooth);
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
