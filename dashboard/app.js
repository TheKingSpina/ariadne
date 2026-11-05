/* Read-only SVG visualization of a verified L1 projection. */
function renderGraph(graph, output, context = {}) {
  const nodes = Array.isArray(graph.nodes) ? graph.nodes : [];
  const edges = Array.isArray(graph.edges) ? graph.edges : [];
  const active = context.active instanceof Set ? context.active : new Set(context.active || []);
  const memory = context.memory instanceof Map ? context.memory : new Map();
  const title = document.createElement('div');
  title.className = 'graph-heading';
  const summary = document.createElement('strong');
  summary.textContent = `${nodes.length} ${nodes.length === 1 ? 'nodo' : 'nodi'} · ${edges.length} ${edges.length === 1 ? 'arco' : 'archi'}`;
  const origin = document.createElement('span');
  const register = context.register || 'registro corrente';
  const head = graph.head_hash ? String(graph.head_hash).slice(0, 16) + '…' : 'testa non disponibile';
  origin.textContent = `Derivata da ${register} · testa ${head}`;
  title.append(summary, origin); output.append(title);
  if (!nodes.length) {
    const empty = document.createElement('div'); empty.className='graph-empty';
    empty.textContent='Nessun nodo nel grafo di questo registro.'; output.append(empty); return;
  }

  const ns='http://www.w3.org/2000/svg';
  const svgEl = (name, attrs={}) => { const el=document.createElementNS(ns,name); for(const [k,v] of Object.entries(attrs)) el.setAttribute(k,String(v)); return el; };
  const kindLabel = node => `${node.kind}/${node.key}`;
  const nodeById = new Map(nodes.map(node => [node.id, node]));

  const toolbar = document.createElement('div'); toolbar.className = 'graph-toolbar';
  const search = document.createElement('input');
  search.type = 'search'; search.placeholder = 'Cerca per chiave, tipo o ID'; search.maxLength = 200;
  search.setAttribute('aria-label', 'Filtra i nodi del grafo');
  const kind = document.createElement('select'); kind.setAttribute('aria-label', 'Filtra per tipo');
  const allKinds = [...new Set(nodes.map(node => node.kind))].sort();
  const allOption = document.createElement('option'); allOption.value = ''; allOption.textContent = 'Tutti i tipi'; kind.append(allOption);
  for (const value of allKinds) { const option = document.createElement('option'); option.value = value; option.textContent = value; kind.append(option); }
  const zoomOut = document.createElement('button'); zoomOut.type='button'; zoomOut.className='secondary compact'; zoomOut.textContent='−'; zoomOut.setAttribute('aria-label','Riduci lo zoom');
  const zoomIn = document.createElement('button'); zoomIn.type='button'; zoomIn.className='secondary compact'; zoomIn.textContent='+'; zoomIn.setAttribute('aria-label','Aumenta lo zoom');
  const zoomReset = document.createElement('button'); zoomReset.type='button'; zoomReset.className='secondary compact'; zoomReset.textContent='100%'; zoomReset.setAttribute('aria-label','Ripristina lo zoom');
  const count = document.createElement('span'); count.className = 'graph-filter-count';
  toolbar.append(search, kind, zoomOut, zoomIn, zoomReset, count);
  const legend=document.createElement('div'); legend.className='graph-legend';
  for (const [cls,label] of [['l2-active','L2 attivo'],['l3-memoria-lunga','L3 memoria lunga'],['l3-archiviata','L3 archiviata']]) {
    const item=document.createElement('span'); const dot=document.createElement('i'); dot.className='legend-dot '+cls;
    item.append(dot, document.createTextNode(label)); legend.append(item);
  }

  const viewport=document.createElement('div'); viewport.className='graph-viewport';
  const svg=svgEl('svg', {role:'img', 'aria-label':`Grafo con ${nodes.length} nodi e ${edges.length} archi`});
  const details=document.createElement('div'); details.className='graph-details';
  details.textContent='Seleziona un nodo per vedere tipo, ID, origine e relazioni.';
  let zoom = 1, selectedId = null;

  function visibleNodes() {
    const text = search.value.trim().toLowerCase();
    const wantedKind = kind.value;
    return nodes.filter(node => {
      if (wantedKind && node.kind !== wantedKind) return false;
      if (!text) return true;
      return `${node.kind} ${node.key} ${node.id}`.toLowerCase().includes(text);
    });
  }

  function relationsFor(node) {
    const outgoing = edges.filter(edge => edge.src_id === node.id).map(edge => ({rel: edge.rel, other: nodeById.get(edge.dst_id), origin: edge.origin_seq, direction: 'out'}));
    const incoming = edges.filter(edge => edge.dst_id === node.id && edge.src_id !== node.id).map(edge => ({rel: edge.rel, other: nodeById.get(edge.src_id), origin: edge.origin_seq, direction: 'in'}));
    return {outgoing, incoming};
  }

  function showNode(node) {
    selectedId = node.id;
    details.replaceChildren();
    const heading=document.createElement('strong'); heading.textContent=kindLabel(node);
    const meta=document.createElement('div'); meta.textContent=`Origine L0: seq ${node.origin_seq}`;
    const id=document.createElement('code'); id.textContent=node.id;
    const registerLine=document.createElement('div'); registerLine.className='graph-relation-meta'; registerLine.textContent=`Vista derivata da ${register} · testa ${head}`;
    details.append(heading, meta, id, registerLine);
    const state=memory.get(node.id);
    const status=document.createElement('div'); status.className='graph-status';
    const l2=document.createElement('span'); l2.className='status-chip '+(active.has(node.id)?'on':'off');
    l2.textContent=active.has(node.id) ? 'L2 attivo' : 'L2 non attivo';
    const l3=document.createElement('span'); l3.className='status-chip '+(state ? String(state.status).replace('_','-') : 'off');
    l3.textContent='L3 ' + (state ? state.status + (state.archived_at_seq !== undefined ? ` @${state.archived_at_seq}` : '') : 'non tracciato');
    status.append(l2,l3); details.append(status);
    const {outgoing, incoming} = relationsFor(node);
    const list=document.createElement('ul'); list.className='graph-relations';
    for (const item of outgoing) {
      const entry=document.createElement('li');
      const label=document.createElement('span'); label.textContent=item.direction === 'out' ? `${item.rel} →` : `← ${item.rel}`;
      const target=document.createElement('span'); target.textContent=item.other ? kindLabel(item.other) : '(nodo non presente)';
      if (item.other) { target.className='graph-relation-link'; target.tabIndex=0; target.setAttribute('role','button'); target.addEventListener('click',()=>showNode(item.other)); target.addEventListener('keydown',event=>{if(event.key==='Enter'||event.key===' '){event.preventDefault();showNode(item.other);}}); }
      const seq=document.createElement('small'); seq.textContent=`origine ${item.origin}`;
      entry.append(label,target,seq); list.append(entry);
    }
    for (const item of incoming) {
      const entry=document.createElement('li');
      const label=document.createElement('span'); label.textContent=`← ${item.rel}`;
      const target=document.createElement('span'); target.textContent=item.other ? kindLabel(item.other) : '(nodo non presente)';
      if (item.other) { target.className='graph-relation-link'; target.tabIndex=0; target.setAttribute('role','button'); target.addEventListener('click',()=>showNode(item.other)); target.addEventListener('keydown',event=>{if(event.key==='Enter'||event.key===' '){event.preventDefault();showNode(item.other);}}); }
      const seq=document.createElement('small'); seq.textContent=`origine ${item.origin}`;
      entry.append(label,target,seq); list.append(entry);
    }
    if (!list.children.length) { const none=document.createElement('div'); none.className='graph-relation-meta'; none.textContent='Nessuna relazione per questo nodo.'; details.append(none); }
    else { const label=document.createElement('div'); label.className='head-label'; label.textContent='RELAZIONI'; details.append(label, list); }
    svg.querySelectorAll('.graph-node').forEach(el=>el.classList.toggle('selected',el.dataset.nodeId===node.id));
  }

  function draw() {
    const visible = visibleNodes();
    const visibleIds = new Set(visible.map(node => node.id));
    count.textContent = `${visible.length} di ${nodes.length} nodi visibili`;
    svg.replaceChildren();
    const defs=svgEl('defs'), marker=svgEl('marker',{id:'graph-arrow',viewBox:'0 0 10 10',refX:9,refY:5,markerWidth:6,markerHeight:6,orient:'auto-start-reverse'});
    marker.append(svgEl('path',{d:'M 0 0 L 10 5 L 0 10 z',fill:'#83a28a'})); defs.append(marker); svg.append(defs);
    if (!visible.length) {
      svg.setAttribute('viewBox','-340 -340 680 680'); svg.style.width='680px'; svg.style.height='680px';
      const empty=svgEl('text',{x:0,y:0,'text-anchor':'middle','class':'graph-node-label'});
      empty.textContent='Nessun nodo corrisponde al filtro'; svg.append(empty); return;
    }
    const positions=new Map(); const groups=[]; let remaining=visible.length, radius=0;
    if (visible.length === 1) groups.push({radius:0,count:1});
    else while (remaining>0) {
      radius += 120; const ringCount=Math.min(remaining,Math.max(8,Math.floor(2*Math.PI*radius/105)));
      groups.push({radius,count:ringCount}); remaining-=ringCount;
    }
    const extent=Math.max(340,radius+100), size=extent*2;
    svg.setAttribute('viewBox',`${-extent} ${-extent} ${size} ${size}`);
    svg.style.width=`${Math.max(680,size)*zoom}px`;
    svg.style.height=`${Math.max(680,size)*zoom}px`;
    let index=0;
    for(const ring of groups) for(let i=0;i<ring.count;i++) {
      const node=visible[index++], angle=-Math.PI/2+2*Math.PI*i/ring.count;
      positions.set(node.id,{x:ring.radius*Math.cos(angle),y:ring.radius*Math.sin(angle),node});
    }
    const edgesLayer=svgEl('g',{'class':'graph-edges'}), nodesLayer=svgEl('g',{'class':'graph-nodes'});
    let hidden=0;
    for(const edge of edges) {
      if (!visibleIds.has(edge.src_id) || !visibleIds.has(edge.dst_id)) {hidden++;continue;}
      const a=positions.get(edge.src_id), b=positions.get(edge.dst_id);
      if (!a || !b) {hidden++;continue;}
      const path=svgEl('path',{'class':'graph-edge','marker-end':'url(#graph-arrow)'});
      let labelX, labelY;
      if (edge.src_id === edge.dst_id) {
        path.setAttribute('d',`M ${a.x-14} ${a.y-19} C ${a.x-58} ${a.y-92}, ${a.x+58} ${a.y-92}, ${a.x+14} ${a.y-19}`);
        labelX=a.x; labelY=a.y-75;
      } else {
        const dx=b.x-a.x,dy=b.y-a.y,length=Math.hypot(dx,dy)||1, ux=dx/length,uy=dy/length;
        const x1=a.x+ux*25,y1=a.y+uy*25,x2=b.x-ux*29,y2=b.y-uy*29;
        const bend=18, mx=(x1+x2)/2-uy*bend,my=(y1+y2)/2+ux*bend;
        path.setAttribute('d',`M ${x1} ${y1} Q ${mx} ${my} ${x2} ${y2}`); labelX=mx; labelY=my-5;
      }
      const label=svgEl('text',{'class':'graph-edge-label',x:labelX,y:labelY,'text-anchor':'middle'});
      label.textContent=edge.rel; edgesLayer.append(path,label);
    }
    for(const {x,y,node} of positions.values()) {
      const state=memory.get(node.id);
      const classes=['graph-node'];
      if (active.has(node.id)) classes.push('l2-active');
      if (state) classes.push('l3-'+String(state.status).replace('_','-'));
      if (node.id===selectedId) classes.push('selected');
      const group=svgEl('g',{'class':classes.join(' '),transform:`translate(${x} ${y})`,tabindex:'0',role:'button','aria-label':`Nodo ${node.kind}: ${node.key}`});
      group.dataset.nodeId=node.id;
      const circle=svgEl('circle',{r:23}); const symbol=svgEl('text',{'text-anchor':'middle',dy:'0.35em','class':'graph-node-symbol'}); symbol.textContent='✳';
      const label=svgEl('text',{'text-anchor':'middle',y:42,'class':'graph-node-label'});
      label.textContent=node.key.length>26 ? node.key.slice(0,25)+'…' : node.key;
      const titleEl=svgEl('title'); titleEl.textContent=`${node.kind}: ${node.key}`;
      group.append(circle,symbol,label,titleEl); group.addEventListener('click',()=>showNode(node));
      group.addEventListener('keydown',event=>{if(event.key==='Enter'||event.key===' '){event.preventDefault();showNode(node);}});
      nodesLayer.append(group);
    }
    svg.append(edgesLayer,nodesLayer);
    if (hidden) count.textContent += ` · ${hidden} archi nascosti dal filtro`;
  }

  const rerender = () => draw();
  search.addEventListener('input', rerender);
  kind.addEventListener('change', rerender);
  zoomOut.addEventListener('click', () => { zoom = Math.max(0.5, zoom / 1.25); draw(); });
  zoomIn.addEventListener('click', () => { zoom = Math.min(3, zoom * 1.25); draw(); });
  zoomReset.addEventListener('click', () => { zoom = 1; draw(); });

  viewport.append(svg); output.append(toolbar, legend, viewport, details);
  const raw=document.createElement('details'); raw.className='graph-raw';
  const toggle=document.createElement('summary'); toggle.textContent='Mostra dati JSON verificati';
  const pre=document.createElement('pre'); pre.className='json-view'; pre.textContent=JSON.stringify(graph,null,2);
  raw.append(toggle,pre); output.append(raw);
  draw();
}

const state = { logs: [], selected: null, data: null, view: 'events' };
const $ = id => document.getElementById(id);
async function api(path) {
  const response = await fetch(path, {cache: 'no-store'});
  const data = await response.json();
  if (!response.ok) throw new Error(data.error || 'Errore del servizio');
  return data;
}
function error(message) { const el = $('result'); el.textContent = message; el.className = 'result error'; }

async function refreshDiagnostics() {
  try {
    const data = await api('/api/diagnostics');
    const archive = data.archive || {}, mcp = data.mcp || {};
    const healthy = archive.reachable && mcp.reachable;
    $('diag-pill').textContent = healthy ? 'SERVIZI RAGGIUNGIBILI' : 'DEGRADATO';
    $('diag-pill').className = 'pill' + (healthy ? '' : ' muted');
    $('diag-archive').textContent = archive.reachable ? `${archive.logs} registri · ${archive.events} eventi` : 'non raggiungibile';
    $('diag-mcp').textContent = mcp.reachable ? `raggiungibile · ${mcp.sessions ?? 0} sessioni` : `non raggiungibile (${mcp.error || 'errore'})`;
    $('diag-updated').textContent = archive.updated_at || '—';
    $('diag-versions').textContent = (data.protocol_versions || []).map(v => `v${v}`).join(', ') || '—';
    $('diag-raw').textContent = JSON.stringify(data, null, 2);
  } catch (e) {
    $('diag-pill').textContent = 'DIAGNOSTICA NON DISPONIBILE';
    $('diag-pill').className = 'pill muted';
    $('diag-archive').textContent = 'non disponibile';
    $('diag-mcp').textContent = 'stato sconosciuto';
  }
}

async function refresh() {
  try {
    await Promise.all([refreshMemories(), refreshDiagnostics()]);
    const data = await api('/api/logs'); state.logs = data.logs;
    $('log-count').textContent = state.logs.length;
    $('count-pill').textContent = state.logs.length;
    $('event-count').textContent = state.logs.reduce((sum, row) => sum + row.events, 0);
    const list = $('log-list'); list.replaceChildren();
    if (!state.logs.length) { const empty = document.createElement('div'); empty.className='empty'; empty.textContent='Nessun registro salvato.'; list.append(empty); }
    for (const item of state.logs) {
      const button = document.createElement('button'); button.className = 'log-item' + (item.id === state.selected ? ' active' : '');
      const title = document.createElement('strong'); title.textContent = item.id;
      const meta = document.createElement('small'); meta.textContent = `v${item.version} · ${item.events} eventi`;
      button.append(title, meta); button.addEventListener('click', () => select(item.id)); list.append(button);
    }
    if (state.selected && state.logs.some(row => row.id === state.selected)) await select(state.selected);
  } catch (e) { $('log-list').textContent = 'Archivio non disponibile: ' + e.message; }
}
async function refreshMemories() {
  const target=$('memory-list');
  try {
    const params = new URLSearchParams({q: $('memory-query').value, limit: '50'});
    if ($('memory-tag').value.trim()) params.set('tag', $('memory-tag').value.trim());
    if ($('memory-source').value.trim()) params.set('source', $('memory-source').value.trim());
    if ($('memory-from').value) params.set('from', $('memory-from').value);
    if ($('memory-to').value) params.set('to', $('memory-to').value);
    if ($('memory-current').checked) params.set('current', 'true');
    const data=await api('/api/memories?' + params.toString());
    target.replaceChildren();
    if (!data.memories.length) {const empty=document.createElement('div');empty.className='empty';empty.textContent='Nessun ricordo trovato.';target.append(empty);}
    for(const memory of data.memories) {
      const row=document.createElement('div');row.className='memory-item';
      const content=document.createElement('p');content.textContent=memory.content;
      const details=document.createElement('small');details.textContent=`#${memory.id} · ${memory.created_at}`+(memory.source ? ` · Fonte dichiarata: ${memory.source}` : '')+(memory.tags.length ? ` · ${memory.tags.join(', ')}` : '');
      row.append(content,details);
      const links=document.createElement('small'); links.className='memory-links';
      if (memory.supersedes) links.textContent=`Rettifica di #${memory.supersedes}`;
      if (memory.superseded_by) links.textContent=(links.textContent ? links.textContent+' · ' : '')+`Superata da #${memory.superseded_by}`;
      if (links.textContent) row.append(links);
      target.append(row);
    }
  } catch(e) {target.textContent='Memorie non disponibili: '+e.message;}
}
async function select(id) {
  try {
    state.data = await api('/api/logs/' + encodeURIComponent(id)); state.selected=id;
    $('detail-title').textContent=id; $('active-name').textContent=id;
    $('active-detail').textContent=`Protocollo v${state.data.version}`; $('version-pill').textContent=`PROTOCOLLO V${state.data.version}`;
    $('detail-empty').hidden=true; $('detail-content').hidden=false;
    const events=state.data.log_jsonl.trimEnd().split('\n').map(line => JSON.parse(line));
    $('head-hash').textContent=events.length ? events[events.length-1].hash : '—';
    document.querySelectorAll('.log-item').forEach(button => button.classList.toggle('active', button.firstChild.textContent === id));
    await render();
  } catch (e) { error(e.message); }
}
async function render() {
  if (!state.selected) return;
  const selected = state.selected, view = state.view;
  const output=$('result'); output.className='result'; output.replaceChildren();
  try {
    if (state.view === 'events') {
      const events=state.data.log_jsonl.trimEnd().split('\n').map(line => JSON.parse(line));
      for (const event of events.reverse()) {
        const row=document.createElement('div'); row.className='event';
        const top=document.createElement('div'); top.className='event-top';
        const tag=document.createElement('span'); tag.className='event-type'; tag.textContent=event.type;
        const info=document.createElement('small'); info.textContent=`SEQ ${event.seq} · TICK ${event.tick}`;
        const payload=document.createElement('pre'); payload.textContent=JSON.stringify(event.payload,null,2);
        top.append(tag,info); row.append(top,payload); output.append(row);
      }
    } else {
      let data, context = {register: selected};
      if (view === 'graph' && state.data.version === '2') {
        const [graph, active, memory] = await Promise.all([
          api(`/api/logs/${encodeURIComponent(selected)}/views/graph`),
          api(`/api/logs/${encodeURIComponent(selected)}/views/active`),
          api(`/api/logs/${encodeURIComponent(selected)}/views/memory`)
        ]);
        data = graph;
        context.active = new Set(active.ids || []);
        context.memory = new Map((memory.entries || []).map(entry => [entry.id, entry]));
      } else {
        data = await api(`/api/logs/${encodeURIComponent(selected)}/views/${encodeURIComponent(view)}`);
      }
      if (state.selected !== selected || state.view !== view) return;
      if (view === 'graph') renderGraph(data, output, context);
      else { const pre=document.createElement('pre'); pre.className='json-view'; pre.textContent=JSON.stringify(data,null,2); output.append(pre); }
    }
  } catch (e) { error(e.message); }
}
document.querySelectorAll('.tab').forEach(button => button.addEventListener('click', () => {
  state.view=button.dataset.view; document.querySelectorAll('.tab').forEach(tab=>tab.classList.toggle('selected',tab===button)); render();
}));
$('refresh').addEventListener('click',refresh);
$('memory-search').addEventListener('submit',event=>{event.preventDefault();refreshMemories();});
refresh();
// The server pushes an initial resync and notifications after committed writes.
// EventSource reconnects automatically after network or container restarts.
const changes = new EventSource('/api/stream');
changes.addEventListener('change', () => refresh());
