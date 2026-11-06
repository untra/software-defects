import * as vscode from 'vscode';
import { EVIDENCE_COMMAND, explanation } from './hover';
import { LanguageClient, LSPErrorCodes, ResponseError, State } from 'vscode-languageclient/node';
import { accepts, compare, contains, equal, functionMarkers, isImpure, Entry, Evidence, segments, triggerSegments, Snapshot, Range, decorations as paint, PALETTE, Category } from './ranges';

const LANGUAGES = ['go', 'rust', 'typescript', 'python'];
const REFRESH_DELAY_MS = 180;
const ANALYSIS_TIMEOUT_MS = 10000;
const LOG_LIMIT = 200;
const CONFIG_SECTION = 'stateLens';
const SERVER_ENV = 'STATE_LENS_SERVER';
const SERVER_BINARY = 'state-lens';
const INLAY_MARKER = '◆';
const BORDER_WIDTH = { source: 1, accessible: 2, function: 2, compound: 2 } as const;
let client: LanguageClient | undefined;

export interface Applied { version: number; findings: number; segments: number; triggers: number; functionMarkers: Range[]; decorations: { category: Category; range: Range; outline: boolean; families: string[] }[] }
export interface AuditSnapshot extends Applied { audit: Snapshot }
export interface StateLensApi {
  log(): string[];
  snapshot(uri: string): Promise<AuditSnapshot | undefined>;
  serverRunning(): boolean;
  status(): string;
  applied(uri: string): Applied | undefined;
}

function range(value: Range): vscode.Range { return new vscode.Range(value.start.line, value.start.character, value.end.line, value.end.character); }
type EvidenceNode = vscode.TreeItem & { children?: vscode.TreeItem[]; entry?: Entry };
class EvidenceView implements vscode.TreeDataProvider<vscode.TreeItem> {
  private emitter = new vscode.EventEmitter<void>();
  readonly onDidChangeTreeData = this.emitter.event;
  private snapshots = new Map<string, Snapshot>();
  private selected?: Entry;
  private roots?: EvidenceNode[];
  snapshot(uri: string): Snapshot | undefined { return this.snapshots.get(uri); }
  select(entry: Entry): EvidenceNode | undefined {
    this.selected = entry;
    this.roots = undefined;
    const item = this.getChildren().find(item => item.entry === entry);
    this.emitter.fire();
    return item;
  }
  update(snapshot: Snapshot): void { this.snapshots.set(snapshot.uri, snapshot); this.roots = undefined; this.emitter.fire(); }
  clear(): void { this.snapshots.clear(); this.selected = undefined; this.roots = undefined; this.emitter.fire(); }
  getTreeItem(item: vscode.TreeItem): vscode.TreeItem { return item; }
  getParent(item: EvidenceNode): EvidenceNode | undefined { return this.getChildren().find(root => root.children?.includes(item)); }
  getChildren(element?: EvidenceNode): EvidenceNode[] {
    if (element) return element.children ?? [];
    return this.roots ??= [...this.snapshots.values()].flatMap(snapshot => snapshot.findings.map(entry => {
      const item: EvidenceNode = new vscode.TreeItem(entry.finding.defect, entry === this.selected ? vscode.TreeItemCollapsibleState.Expanded : vscode.TreeItemCollapsibleState.Collapsed);
      item.entry = entry;
      const metadata = [
        ['Origin', entry.finding.origin, entry.finding.origin === 'direct' ? 'Detected in this function.' : 'Arrives through a resolved call.'],
        ['Effect', entry.finding.effect.kind, effectMeaning(entry.finding.effect.kind)],
        ['Resolution', entry.finding.resolution, resolutionMeaning(entry.finding.resolution)],
        ['Taxonomy', entry.finding.related_cwes?.join(', ') || 'None', 'Related taxonomy references; these are not diagnoses.']
      ];
      item.children = metadata.map(([label, value, tooltip]) => {
        const child = new vscode.TreeItem(`${label}: ${value}`);
        child.tooltip = tooltip;
        return child;
      }).concat([...entry.finding.call_path, ...entry.finding.evidence, ...(entry.finding.compound?.ingredients ?? [])].map(evidence => {
        const child = new vscode.TreeItem(`${vscode.workspace.asRelativePath(vscode.Uri.parse(evidence.uri))}: bytes ${evidence.range.start}–${evidence.range.end}`);
        child.command = { command: 'stateLens.openEvidence', title: 'Open originating evidence', arguments: [snapshot.uri, snapshot.version, evidence] };
        return child;
      }));
      item.description = `${vscode.workspace.asRelativePath(vscode.Uri.parse(snapshot.uri))}:${entry.triggerRange.start.line + 1}`;
      item.tooltip = explanation(entry);
      item.command = { command: EVIDENCE_COMMAND, title: 'Show evidence', arguments: [snapshot.uri, snapshot.version, entry.finding] };
      return item;
    }));
  }
  dispose(): void { this.emitter.dispose(); }
}

function effectMeaning(kind: string): string {
  const meanings: Record<string, string> = {
    local_mutation: 'Mutable binding or local write. Marks this function Impure without propagating to callers.',
    argument_mutation: 'Changes a value supplied by a caller.', receiver_mutation: 'Changes receiver state.',
    global_mutation: 'Changes state outside this function.', external_read: 'Reads mutable state outside this function.',
    nondeterminism: 'Depends on time, randomness, or another modeled nondeterministic source.',
    structural_signal: 'A code structure requiring review; alone it does not establish impurity.'
  };
  return meanings[kind] ?? kind;
}
function resolutionMeaning(resolution: string): string {
  const meanings: Record<string, string> = {
    syntax: 'Recognized from code syntax.', 'binding-analysis': 'Resolved using lexical bindings.',
    'modeled-api': 'Recognized using a supported API model.', candidate: 'A candidate whose behavior is not proven.',
    'resolved-call': 'Resolved to a known function.'
  };
  return meanings[resolution] ?? `Analyzer resolution: ${resolution}.`;
}

export async function activate(context: vscode.ExtensionContext): Promise<StateLensApi> {
  const evidence = new EvidenceView();
  const inlayChanges = new vscode.EventEmitter<void>();
  context.subscriptions.push(inlayChanges);
  const evidenceTree = vscode.window.createTreeView('stateLens.evidence', { treeDataProvider: evidence });
  context.subscriptions.push(evidence, evidenceTree);
  const channel = vscode.window.createOutputChannel('State Lens');
  const recorded: string[] = [];
  let partial = '';
  const record = (value: string): void => {
    const lines = (partial + value).split('\n');
    partial = lines.pop() ?? '';
    recorded.push(...lines);
    if (recorded.length > LOG_LIMIT) recorded.splice(0, recorded.length - LOG_LIMIT);
  };
  const output: vscode.OutputChannel = {
    name: channel.name,
    append: value => { record(value); channel.append(value); },
    appendLine: value => { record(value + '\n'); channel.appendLine(value); },
    replace: value => { record(value + '\n'); channel.replace(value); },
    clear: () => channel.clear(),
    show: (column?: vscode.ViewColumn | boolean, preserveFocus?: boolean) => {
      if (typeof column === 'boolean') channel.show(column);
      else channel.show(column, preserveFocus);
    },
    hide: () => channel.hide(),
    dispose: () => channel.dispose()
  };
  const status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right);
  status.command = 'stateLens.toggle';
  context.subscriptions.push(output, status);
  let epoch = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let running = false;
  let rerun = false;
  let disposed = false;
  const functionStyle = vscode.window.createTextEditorDecorationType({
    borderWidth: `0 0 0 ${BORDER_WIDTH.function}px`, borderStyle: 'solid', borderColor: PALETTE['mutated-state'],
    rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed
  });
  context.subscriptions.push(functionStyle);
  let decorations: vscode.TextEditorDecorationType[] = [];
  let stylesByCategory: { category: Category; propagated: boolean; type: vscode.TextEditorDecorationType }[] = [];
  const applied = new Map<string, Applied>();
  async function requestSnapshot(uri: string): Promise<Snapshot> {
    if (!client?.isRunning()) throw new Error('State Lens server unavailable');
    const started = Date.now();
    const cancellation = new vscode.CancellationTokenSource();
    let timer: ReturnType<typeof setTimeout> | undefined;
    try {
      const snapshot = await Promise.race([
        client.sendRequest<Snapshot>('stateLens/ranges', { textDocument: { uri } }, cancellation.token),
        new Promise<never>((_, reject) => {
          timer = setTimeout(() => {
            reject(new Error(`Analysis exceeded ${ANALYSIS_TIMEOUT_MS}ms for ${uri}`));
            cancellation.cancel();
          }, ANALYSIS_TIMEOUT_MS);
        })
      ]);
      output.appendLine(`Analysis ${uri}: version=${snapshot.version} findings=${snapshot.findings.length} indexing=${snapshot.indexing} elapsed=${Date.now() - started}ms`);
      return snapshot;
    } catch (error) {
      if (!(error instanceof ResponseError && error.code === LSPErrorCodes.ContentModified)) output.appendLine(`[Error] Analysis ${uri} after ${Date.now() - started}ms: ${String(error)}`);
      throw error;
    } finally {
      if (timer) clearTimeout(timer);
      cancellation.dispose();
    }
  }
  const api: StateLensApi = {
    log: () => [...recorded, ...(partial ? [partial] : [])],
    snapshot: async uri => {
      const document = vscode.workspace.textDocuments.find(doc => doc.uri.toString() === uri);
      if (!document || !client?.isRunning()) throw new Error('State Lens document or server unavailable');
      const requestedEpoch = epoch;
      const version = document.version;
      try {
        const snapshot = await requestSnapshot(uri);
        if (disposed || !accepts(snapshot, uri, document.version, requestedEpoch, epoch) || version !== document.version) return undefined;
        const directOnly = config().get<boolean>('directOnly', false);
        return { version, findings: snapshot.findings.length, segments: segments(snapshot, directOnly).length,
          triggers: triggerSegments(snapshot, directOnly).length, functionMarkers: functionMarkers(snapshot, directOnly).map(marker => marker.range), decorations: paint(snapshot, directOnly).map(({ category, range, outline, families }) => ({ category, range, outline, families })), audit: snapshot };
      } catch (error) {
        if (error instanceof ResponseError && error.code === LSPErrorCodes.ContentModified) return undefined;
        throw error;
      }
    },
    serverRunning: () => client?.isRunning() ?? false,
    status: () => status.text,
    applied: uri => applied.get(uri)
  };
  const config = () => vscode.workspace.getConfiguration(CONFIG_SECTION);
  function styles(): void {
    decorations.forEach(d => d.dispose());
    const opacity = config().get<number>('opacity', 0.10);
    stylesByCategory = (Object.entries(PALETTE) as [Category, string][]).flatMap(([category, hex]) => {
      const rgb = [1, 3, 5].map(offset => parseInt(hex.slice(offset, offset + 2), 16)).join(', ');
      const outline = !category.endsWith('-state');
      return [false, true].map(propagated => {
        const alpha = propagated ? opacity / 2 : opacity;
        return { category, propagated, type: vscode.window.createTextEditorDecorationType({
          backgroundColor: outline ? undefined : `rgba(${rgb}, ${alpha})`,
          border: outline ? `${BORDER_WIDTH.compound}px solid` : `${config().get<boolean>('accessibleMarkers') ? BORDER_WIDTH.accessible : BORDER_WIDTH.source}px ${propagated ? 'dashed' : 'dotted'}`,
          borderColor: `rgba(${rgb}, ${propagated ? 0.5 : 1})`,
          rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
          isWholeLine: false,
          overviewRulerColor: hex
        }) };
      });
    });
    decorations = stylesByCategory.map(style => style.type);
  }
  function clear(): void {
    for (const editor of vscode.window.visibleTextEditors) {
      decorations.forEach(d => editor.setDecorations(d, []));
      editor.setDecorations(functionStyle, []);
    }
    applied.clear();
    evidence.clear();
    inlayChanges.fire();
  }
  function schedule(): void {
    epoch++;
    clear();
    if (disposed) return;
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => { void refresh(); }, REFRESH_DELAY_MS);
  }
  async function refresh(): Promise<void> {
    if (disposed || !client?.isRunning()) return;
    if (running) { rerun = true; return; }
    if (!config().get<boolean>('enabled', true)) { clear(); status.text = 'State Lens: off'; return; }
    running = true;
    const requestedEpoch = epoch;
    status.text = 'State Lens: analyzing…';
    try {
      const editors = vscode.window.visibleTextEditors.filter(e => LANGUAGES.includes(e.document.languageId) && e.document.uri.scheme === 'file');
      let incomplete = false;
      for (const editor of editors) {
        const uri = editor.document.uri.toString();
        const snapshot = await requestSnapshot(uri);
        if (disposed || !accepts(snapshot, uri, editor.document.version, requestedEpoch, epoch)) continue;
        const directOnly = config().get<boolean>('directOnly', false);
        const parts = segments(snapshot, directOnly);
        const painted = paint(snapshot, directOnly);
        for (const style of stylesByCategory) {
          editor.setDecorations(style.type, painted.filter(p => p.category === style.category && p.propagated === style.propagated).map(p => ({
            range: range(p.range),
            renderOptions: p.families.length > 1 ? { after: { contentText: ` [${p.families.join(' + ')}]`, color: PALETTE[p.category] } } : undefined
          })));
        }
        const markers = functionMarkers(snapshot, directOnly);
        editor.setDecorations(functionStyle, markers.map(marker => range(marker.range)));
        const triggers = triggerSegments(snapshot, directOnly);
        applied.set(uri, { version: editor.document.version, findings: snapshot.findings.length, segments: parts.length, triggers: triggers.length, functionMarkers: markers.map(marker => marker.range),
          decorations: painted.map(({ category, range, outline, families }) => ({ category, range, outline, families })) });
        evidence.update({ ...snapshot, findings: snapshot.findings.filter(e => !directOnly || e.finding.origin === 'direct') });
        inlayChanges.fire();
        incomplete ||= Boolean(snapshot.indexing) || snapshot.parseErrors || snapshot.unresolved.length > 0;
      }
      if (requestedEpoch === epoch) status.text = `State Lens: active${incomplete ? ' (partial)' : ''}`;
    } catch (error) {
      if (error instanceof ResponseError && error.code === LSPErrorCodes.ContentModified) { rerun = true; return; }
      clear();
      status.text = 'State Lens: unavailable';
      status.tooltip = `${String(error)}\nRun State Lens: Show Debug Information for server details and logs.`;
      output.appendLine(`[Error] ${String(error)}`);
    } finally {
      running = false;
      if (rerun || epoch !== requestedEpoch) { rerun = false; schedule(); }
    }
  }
  styles();
  context.subscriptions.push({ dispose: () => { disposed = true; epoch++; clear(); if (timer) clearTimeout(timer); decorations.forEach(d => d.dispose()); } });
  context.subscriptions.push(
    vscode.commands.registerCommand('stateLens.toggle', async () => { await config().update('enabled', !config().get<boolean>('enabled', true), vscode.ConfigurationTarget.Global); }),
    vscode.commands.registerCommand('stateLens.refresh', schedule),
    vscode.commands.registerCommand('stateLens.debug', async () => {
      const document = await vscode.workspace.openTextDocument({ language: 'json', content: JSON.stringify({
        extensionVersion: context.extension.packageJSON.version,
        configuredServer: config().get<string>('serverPath'),
        environmentServer: process.env[SERVER_ENV],
        command, path: process.env.PATH, serverInfo: client?.initializeResult?.serverInfo,
        serverRunning: api.serverRunning(), status: status.text,
        documents: vscode.window.visibleTextEditors.map(editor => ({ uri: editor.document.uri.toString(), language: editor.document.languageId, version: editor.document.version, applied: applied.get(editor.document.uri.toString()) })),
        logs: api.log()
      }, null, 2) });
      await vscode.window.showTextDocument(document);
    }),
    vscode.commands.registerCommand('stateLens.openEvidence', async (uri: unknown, version: unknown, requested: unknown) => {
      if (typeof uri !== 'string' || typeof version !== 'number') return;
      const snapshot = evidence.snapshot(uri);
      const parent = vscode.workspace.textDocuments.find(doc => doc.uri.toString() === uri);
      if (!snapshot || !parent || snapshot.version !== version || parent.version !== version) return;
      const target = snapshot.findings.flatMap(entry => [...entry.finding.call_path, ...entry.finding.evidence, ...(entry.finding.compound?.ingredients ?? [])])
        .find(item => JSON.stringify(item) === JSON.stringify(requested));
      if (!target) return;
      const requestedEpoch = epoch;
      const doc = await vscode.workspace.openTextDocument(vscode.Uri.parse(target.uri));
      if (epoch !== requestedEpoch || evidence.snapshot(uri) !== snapshot || parent.version !== version) return;
      const bytes = Buffer.from(doc.getText(), 'utf8');
      if (target.range.start < 0 || target.range.end > bytes.length || target.range.start > target.range.end) return;
      const start = doc.positionAt(bytes.subarray(0, target.range.start).toString('utf8').length);
      const end = doc.positionAt(bytes.subarray(0, target.range.end).toString('utf8').length);
      await vscode.window.showTextDocument(doc, { selection: new vscode.Range(start, end) });
    }),
    vscode.commands.registerCommand(EVIDENCE_COMMAND, async (uri: unknown, version: unknown, finding: unknown) => {
      if (typeof uri !== 'string' || typeof version !== 'number') return;
      const snapshot = evidence.snapshot(uri);
      const document = vscode.workspace.textDocuments.find(doc => doc.uri.toString() === uri);
      if (!snapshot || !document || snapshot.version !== version || document.version !== version) return;
      const entry = snapshot.findings.find(entry => JSON.stringify(entry.finding) === JSON.stringify(finding));
      if (!entry) return;
      await vscode.window.showTextDocument(document, { selection: range(entry.triggerRange) });
      if (evidence.snapshot(uri) !== snapshot || document.version !== version) return;
      const item = evidence.select(entry);
      if (item) await evidenceTree.reveal(item, { focus: true, select: true, expand: 1 });
    }),
    vscode.languages.registerInlayHintsProvider(LANGUAGES.map(language => ({ scheme: 'file', language })), {
      onDidChangeInlayHints: inlayChanges.event,
      provideInlayHints(document, requestedRange) {
        const snapshot = evidence.snapshot(document.uri.toString());
        if (!snapshot || snapshot.version !== document.version || !config().get<boolean>('enabled', true)) return [];
        return snapshot.functions.flatMap(fn => {
          const entries = snapshot.findings.filter(entry => entry.functionRange && equal(entry.functionRange, fn.range));
          const start = new vscode.Position(fn.body.start.line, fn.body.start.character);
          if (!entries.length || !requestedRange.contains(start)) return [];
          const label = entries.some(isImpure) ? 'Impure' : 'Signals';
          const hint = new vscode.InlayHint(start, `${INLAY_MARKER} ${label}`);
          hint.paddingLeft = true;
          hint.paddingRight = true;
          hint.tooltip = `${fn.name}: ${label}`;
          return [hint];
        });
      }
    }),
    vscode.languages.registerHoverProvider(LANGUAGES.map(language => ({ scheme: 'file', language })), {
      provideHover(document, position) {
        const snapshot = evidence.snapshot(document.uri.toString());
        if (!snapshot || snapshot.version !== document.version || !config().get<boolean>('enabled', true)) return;
        const point = { line: position.line, character: position.character };
        const matches = triggerSegments(snapshot, false).filter(segment => compare(segment.range.start, point) <= 0 && compare(point, segment.range.end) < 0);
        const marker = functionMarkers(snapshot, false).find(marker => contains(marker.range, { start: point, end: point }) && compare(point, marker.range.end) < 0);
        const entries = [...new Set(matches.map(match => match.entry).concat(marker?.entries ?? []))];
        if (!entries.length) return;
        const contents = entries.map(entry => explanation(entry, snapshot.uri, snapshot.version));
        if (marker) {
          const title = new vscode.MarkdownString();
          title.supportHtml = true;
          title.appendMarkdown(`<span style="color:${PALETTE['mutated-state']};"><strong>Impure</strong></span>`);
          contents.unshift(title);
        }
        return new vscode.Hover(contents);
      }
    }),
    vscode.workspace.onDidChangeConfiguration(event => { if (event.affectsConfiguration(CONFIG_SECTION)) { styles(); schedule(); } }),
    vscode.workspace.onDidChangeTextDocument(event => { if (LANGUAGES.includes(event.document.languageId)) schedule(); }),
    vscode.workspace.onDidCloseTextDocument(schedule),
    vscode.window.onDidChangeVisibleTextEditors(schedule)
  );
  const watcher = vscode.workspace.createFileSystemWatcher('**/*.{go,rs,ts,py}');
  context.subscriptions.push(watcher, watcher.onDidCreate(schedule), watcher.onDidChange(schedule), watcher.onDidDelete(schedule));
  const command = config().get<string>('serverPath') || process.env[SERVER_ENV] || SERVER_BINARY;
  output.appendLine(`Starting ${command} serve (extension ${context.extension.packageJSON.version})`);
  client = new LanguageClient(CONFIG_SECTION, 'State Lens', { command, args: ['serve'] }, {
    documentSelector: LANGUAGES.map(language => ({ scheme: 'file', language })),
    initializationOptions: { diagnostics: config().get<boolean>('diagnostics', true) },
    middleware: { provideHover: () => null, provideInlayHints: () => [] },
    synchronize: { fileEvents: watcher, configurationSection: CONFIG_SECTION }, outputChannel: output
  });
  context.subscriptions.push(client.onDidChangeState(event => { if (event.newState === State.Stopped) { epoch++; clear(); status.text = 'State Lens: stopped'; } }));
  context.subscriptions.push(client.onNotification('stateLens/indexing', () => schedule()));
  try { await client.start(); status.show(); schedule(); }
  catch (error) { output.appendLine(`[Error] ${String(error)}`); void vscode.window.showErrorMessage('State Lens could not start. Install the state-lens binary or configure stateLens.serverPath.'); }
  return api;
}
export async function deactivate(): Promise<void> { await client?.stop(); client = undefined; }
