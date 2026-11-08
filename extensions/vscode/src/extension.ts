import { ServerLifecycle, ServerState } from './lifecycle';
import { LANGUAGE, EXTENSIONS, CONFIG_SECTION, commandId } from './package';
import * as vscode from 'vscode';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { createHash } from 'node:crypto';
import { EVIDENCE_COMMAND, explanation } from './hover';
import { LanguageClient, LSPErrorCodes, ResponseError, State, CloseAction, ErrorAction } from 'vscode-languageclient/node';
import { findingEvidence, accepts, compare, contains, equal, functionMarkers, annotations, Entry, Evidence, segments, triggerSegments, Snapshot, Range, decorations as paint, PALETTE, Category, CATEGORY_STYLE, divisionCompatible, StateAnnotation } from './ranges';

const LANGUAGES = [LANGUAGE];
const REFRESH_DELAY_MS = 180;
const ANALYSIS_TIMEOUT_MS = 10000;
const LOG_LIMIT = 200;
const CLEAN_SERVER_EXIT = 'Server process exited with code 0.';
const INLAY_MARKER = '◆';
const ANNOTATION_MARKER_MARGIN = '0 0 0 0.5em';
const ANNOTATION_LABEL_MARGIN = '0 0.5em 0 0.25em';
const BORDER_WIDTH = { source: 1, accessible: 2, function: 2, compound: 2 } as const;
let client: LanguageClient | undefined;
let compatible = true;
let shutdown: (() => Promise<void>) | undefined;

export interface Applied { labels: StateAnnotation[]; version: number; findings: number; segments: number; triggers: number; functionMarkers: Range[]; decorations: { category: Category; range: Range; outline: boolean; families: string[] }[] }
export interface AuditSnapshot extends Applied { audit: Snapshot }
export interface StateLensApi {
  log(after?: number): string[];
  logCount(): number;
  snapshot(uri: string): Promise<AuditSnapshot | undefined>;
  serverRunning(): boolean;
  status(): string;
  applied(uri: string): Applied | undefined;
  ui(): { statusVisible: boolean; evidenceVisible: boolean; evidenceUri?: string; lifecycle: ServerState };
}

function range(value: Range): vscode.Range { return new vscode.Range(value.start.line, value.start.character, value.end.line, value.end.character); }
type EvidenceNode = vscode.TreeItem & { children?: vscode.TreeItem[]; entry?: Entry };
class EvidenceView implements vscode.TreeDataProvider<vscode.TreeItem> {
  private emitter = new vscode.EventEmitter<void>();
  readonly onDidChangeTreeData = this.emitter.event;
  private snapshots = new Map<string, Snapshot>();
  private selected?: Entry;
  private activeUri?: string;
  focus(uri: string | undefined): void {
    if (uri === this.activeUri) return;
    this.activeUri = uri;
    this.selected = undefined;
    this.roots = undefined;
    this.emitter.fire();
  }
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
    return this.roots ??= [...this.snapshots.values()].filter(snapshot => snapshot.uri === this.activeUri).flatMap(snapshot => snapshot.findings.filter(entry => !vscode.workspace.getConfiguration(CONFIG_SECTION).get<boolean>('directOnly', false) || entry.finding.origin === 'direct').map(entry => {
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
      }).concat(findingEvidence(entry.finding).map(evidence => {
        const child = new vscode.TreeItem(`${evidence.role ? evidence.role + ": " : ""}${vscode.workspace.asRelativePath(vscode.Uri.parse(evidence.uri))}: bytes ${evidence.range.start}–${evidence.range.end}`);
        child.command = { command: commandId('openEvidence'), title: 'Open originating evidence', arguments: [snapshot.uri, snapshot.version, evidence] };
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
  const evidenceTree = vscode.window.createTreeView(commandId('evidence'), { treeDataProvider: evidence });
  context.subscriptions.push(evidence, evidenceTree);
  const channel = vscode.window.createOutputChannel(`State Lens (${LANGUAGE})`);
  const recorded: string[] = [];
  let recordedCount = 0;
  let partial = '';
  const record = (value: string): void => {
    const lines = (partial + value).split('\n');
    partial = lines.pop() ?? '';
    recorded.push(...lines);
    recordedCount += lines.length;
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
  status.command = commandId('toggle');
  context.subscriptions.push(output, status);
  let epoch = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let running = false;
  let rerun = false;
  let disposed = false;
  let statusVisible = false;
  const requests = new Set<vscode.CancellationTokenSource>();
  const supports = (editor: vscode.TextEditor | undefined): boolean => editor?.document.languageId === LANGUAGE && editor.document.uri.scheme === 'file';
  function updateUi(): void {
    const active = vscode.window.activeTextEditor;
    statusVisible = !disposed && supports(active);
    evidence.focus(statusVisible ? active?.document.uri.toString() : undefined);
    void vscode.commands.executeCommand('setContext', commandId('activeEditor'), statusVisible);
    status.name = 'State Lens';
    if (!config().get<boolean>('enabled', true)) status.text = 'State Lens: off';
    status.tooltip = `${LANGUAGE}: ${status.text}. Click to toggle State Lens for this language.`;
    if (statusVisible) status.show(); else status.hide();
  }
  const functionStyles = (['mutated-state', 'unstructured-state', 'shared-state'] as const).map(category => ({ category, type: vscode.window.createTextEditorDecorationType({
    before: { contentText: '▌', color: PALETTE[category] }, rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed
  }) }));
  context.subscriptions.push(...functionStyles.map(style => style.type));
  const sourceStyle = vscode.window.createTextEditorDecorationType({ textDecoration: 'underline dotted', rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed });
  context.subscriptions.push(sourceStyle);
  const labelStyle = vscode.window.createTextEditorDecorationType({ rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed });
  context.subscriptions.push(labelStyle);
  let decorations: vscode.TextEditorDecorationType[] = [];
  let stylesByCategory: { category: Category; propagated: boolean; type: vscode.TextEditorDecorationType }[] = [];
  const applied = new Map<string, Applied>();
  async function requestSnapshot(uri: string): Promise<Snapshot> {
    if (!client?.isRunning()) throw new Error('State Lens server unavailable');
    const started = Date.now();
    const cancellation = new vscode.CancellationTokenSource();
    requests.add(cancellation);
    const requestedEpoch = epoch;
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
      if (requestedEpoch === epoch && !(error instanceof ResponseError && error.code === LSPErrorCodes.ContentModified)) output.appendLine(`[Error] Analysis ${uri} after ${Date.now() - started}ms: ${String(error)}`);
      throw error;
    } finally {
      if (timer) clearTimeout(timer);
      requests.delete(cancellation);
      cancellation.dispose();
    }
  }
  const api: StateLensApi = {
    log: (after = 0) => [...recorded.slice(Math.max(0, after - (recordedCount - recorded.length))), ...(partial ? [partial] : [])],
    logCount: () => recordedCount,
    snapshot: async uri => {
      const document = vscode.workspace.textDocuments.find(doc => doc.uri.toString() === uri);
      if (!document || !client?.isRunning()) throw new Error('State Lens document or server unavailable');
      const requestedEpoch = epoch;
      const version = document.version;
      try {
        const snapshot = await requestSnapshot(uri);
        if (disposed || !accepts(snapshot, uri, document.version, requestedEpoch, epoch) || version !== document.version) return undefined;
        const directOnly = config().get<boolean>('directOnly', false);
        return { labels: annotations(snapshot, directOnly), version, findings: snapshot.findings.length, segments: segments(snapshot, directOnly).length,
          triggers: triggerSegments(snapshot, directOnly).length, functionMarkers: functionMarkers(snapshot, directOnly).map(marker => marker.range), decorations: paint(snapshot, directOnly).map(({ category, range, outline, families }) => ({ category, range, outline, families })), audit: snapshot };
      } catch (error) {
        if (error instanceof ResponseError && error.code === LSPErrorCodes.ContentModified) return undefined;
        throw error;
      }
    },
    serverRunning: () => client?.isRunning() ?? false,
    status: () => status.text,
    applied: uri => applied.get(uri),
    ui: () => ({ statusVisible, evidenceVisible: evidenceTree.visible, evidenceUri: statusVisible ? vscode.window.activeTextEditor?.document.uri.toString() : undefined, lifecycle: lifecycle.state })
  };
  const config = () => vscode.workspace.getConfiguration(CONFIG_SECTION);
  function styles(): void {
    decorations.forEach(d => d.dispose());
    const opacity = config().get<number>('opacity', 0.10);
    stylesByCategory = (Object.entries(PALETTE) as [Category, string][]).flatMap(([category, hex]) => {
      const rgb = [1, 3, 5].map(offset => parseInt(hex.slice(offset, offset + 2), 16)).join(', ');
      const outline = CATEGORY_STYLE[category] === 'outline';
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
      editor.setDecorations(labelStyle, []);
      editor.setDecorations(sourceStyle, []);

      decorations.forEach(d => editor.setDecorations(d, []));
      functionStyles.forEach(style => editor.setDecorations(style.type, []));
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
    if (disposed || lifecycle.state !== 'running' || !client?.isRunning()) return;
    if (running) { rerun = true; return; }
    if (!config().get<boolean>('enabled', true)) { clear(); status.text = 'State Lens: off'; return; }
    running = true;
    const requestedEpoch = epoch;
    status.text = 'State Lens: analyzing…';
    try {
      const editors = vscode.window.visibleTextEditors.filter(e => LANGUAGES.includes(e.document.languageId) && e.document.uri.scheme === 'file');
      let incomplete = false;
      const reasons = new Set<string>();
      for (const editor of editors) {
        const uri = editor.document.uri.toString();
        const snapshot = await requestSnapshot(uri);
        if (disposed || !accepts(snapshot, uri, editor.document.version, requestedEpoch, epoch)) continue;
        const directOnly = config().get<boolean>('directOnly', false);
        const parts = segments(snapshot, directOnly);
        const painted = paint(snapshot, directOnly);
        editor.setDecorations(sourceStyle, triggerSegments(snapshot, directOnly).filter(p => !p.entry.finding.compound).map(p => ({ range: range(p.range) })));
        editor.setDecorations(labelStyle, annotations(snapshot, directOnly).map(annotation => ({
          range: new vscode.Range(annotation.position.line, annotation.position.character, annotation.position.line, annotation.position.character),
          hoverMessage: annotation.findings.map(index => snapshot.findings[index].finding.explanation).join('\n'),
          renderOptions: {
            before: { contentText: INLAY_MARKER, color: PALETTE[annotation.category], margin: ANNOTATION_MARKER_MARGIN },
            after: { contentText: annotation.label, color: new vscode.ThemeColor('editor.foreground'), margin: ANNOTATION_LABEL_MARGIN }
          }
        })));
        for (const style of stylesByCategory) {
          editor.setDecorations(style.type, painted.filter(p => p.category === style.category && p.propagated === style.propagated).map(p => ({
            range: range(p.range),
            renderOptions: p.families.length > 1 ? { after: { contentText: ` [${p.families.join(' + ')}]`, color: PALETTE[p.category] } } : undefined
          })));
        }
        const markers = functionMarkers(snapshot, directOnly);
        functionStyles.forEach(style => editor.setDecorations(style.type, markers.filter(marker => marker.category === style.category).map(marker => range(marker.range))));
        const triggers = triggerSegments(snapshot, directOnly);
        applied.set(uri, { labels: annotations(snapshot, directOnly), version: editor.document.version, findings: snapshot.findings.length, segments: parts.length, triggers: triggers.length, functionMarkers: markers.map(marker => marker.range),
          decorations: painted.map(({ category, range, outline, families }) => ({ category, range, outline, families })) });
        evidence.update(snapshot);
        inlayChanges.fire();
        if (snapshot.indexing) reasons.add('Workspace indexing is in progress.');
        if (snapshot.parseErrors) reasons.add('The document has parse errors.');
        snapshot.unresolved.forEach(item => reasons.add(item.reason));
        incomplete ||= Boolean(snapshot.indexing) || snapshot.parseErrors || snapshot.unresolved.length > 0;
      }
      if (requestedEpoch === epoch) { status.text = `State Lens: active${incomplete ? ' (partial)' : ''}`; if (!compatible) status.text = 'State Lens: incompatible server'; status.tooltip = [...reasons, 'Valid findings remain visible. Use State Lens: Show Debug Information for server details.'].join('\n'); }
    } catch (error) {
      if (error instanceof ResponseError && error.code === LSPErrorCodes.ContentModified) { rerun = true; return; }
      if (disposed || requestedEpoch !== epoch) return;
      clear();
      status.text = 'State Lens: unavailable';
      status.tooltip = `${String(error)}\nRun State Lens: Show Debug Information for server details and logs.`;
      output.appendLine(`[Error] ${String(error)}`);
    } finally {
      updateUi();
      running = false;
      if (rerun || epoch !== requestedEpoch) { rerun = false; schedule(); }
    }
  }
  styles();
  context.subscriptions.push({ dispose: () => { void shutdown?.(); decorations.forEach(d => d.dispose()); } });
  context.subscriptions.push(
    vscode.commands.registerCommand(commandId('toggle'), async () => { await config().update('enabled', !config().get<boolean>('enabled', true), vscode.ConfigurationTarget.Global); }),
    vscode.commands.registerCommand(commandId('refresh'), () => reconcile(true)),
    vscode.commands.registerCommand(commandId('debug'), async () => {
      const command = serverCommand();
      const resolvedServer = (path.isAbsolute(command) || command.includes(path.sep) ? [path.resolve(command)] : (process.env.PATH ?? '').split(path.delimiter).map(folder => path.join(folder, command)))
        .find(file => { try { fs.accessSync(file, fs.constants.X_OK); return true; } catch { return false; } });
      let serverHash: string | undefined;
      try { if (resolvedServer) serverHash = createHash('sha256').update(await fs.promises.readFile(resolvedServer)).digest('hex'); } catch {}
      const document = await vscode.workspace.openTextDocument({ language: 'json', content: JSON.stringify({
        extensionVersion: context.extension.packageJSON.version,
        configuredServer: config().get<string>('serverPath'),
        command, resolvedServer, serverHash, path: process.env.PATH, serverInfo: client?.initializeResult?.serverInfo,
        serverRunning: api.serverRunning(), status: status.text,
        documents: vscode.window.visibleTextEditors.map(editor => ({ uri: editor.document.uri.toString(), language: editor.document.languageId, version: editor.document.version, applied: applied.get(editor.document.uri.toString()) })),
        logs: api.log()
      }, null, 2) });
      await vscode.window.showTextDocument(document);
    }),
    vscode.commands.registerCommand(commandId('openEvidence'), async (uri: unknown, version: unknown, requested: unknown) => {
      if (typeof uri !== 'string' || typeof version !== 'number') return;
      const snapshot = evidence.snapshot(uri);
      const parent = vscode.workspace.textDocuments.find(doc => doc.uri.toString() === uri);
      if (!snapshot || !parent || snapshot.version !== version || parent.version !== version) return;
      const target = snapshot.findings.flatMap(entry => findingEvidence(entry.finding))
        .find(item => JSON.stringify(item) === JSON.stringify(requested));
      if (!target) return;
      const requestedEpoch = epoch;
      const doc = await vscode.workspace.openTextDocument(vscode.Uri.parse(target.uri));
      if (epoch !== requestedEpoch || evidence.snapshot(uri) !== snapshot || parent.version !== version) return;
      const bytes = Buffer.from(doc.getText(), 'utf8');
      if (target.source_hash && createHash('sha256').update(bytes).digest('hex') !== target.source_hash) return;
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
    vscode.languages.registerHoverProvider(LANGUAGES.map(language => ({ scheme: 'file', language })), {
      provideHover(document, position) {
        const snapshot = evidence.snapshot(document.uri.toString());
        if (!snapshot || snapshot.version !== document.version || !config().get<boolean>('enabled', true)) return;
        const point = { line: position.line, character: position.character };
        const matches = triggerSegments(snapshot, config().get<boolean>('directOnly', false)).filter(segment => compare(segment.range.start, point) <= 0 && compare(point, segment.range.end) < 0);
        const markers = functionMarkers(snapshot, config().get<boolean>('directOnly', false)).filter(marker => contains(marker.range, { start: point, end: point }) && compare(point, marker.range.end) < 0);
        const entries = [...new Set(matches.map(match => match.entry).concat(markers.flatMap(marker => marker.entries)))];
        if (!entries.length) return;
        const contents = entries.map(entry => explanation(entry, snapshot.uri, snapshot.version));
        for (const marker of markers) {
          const title = new vscode.MarkdownString();
          title.supportHtml = true;
          title.appendMarkdown(`<span style="color:${PALETTE[marker.category]};"><strong>${marker.label}</strong></span>`);
          contents.unshift(title);
        }
        return new vscode.Hover(contents);
      }
    }),
    vscode.workspace.onDidChangeConfiguration(event => { if (event.affectsConfiguration(CONFIG_SECTION)) { styles(); void reconcile(); } }),
    vscode.workspace.onDidChangeTextDocument(event => { if (LANGUAGES.includes(event.document.languageId)) schedule(); }),
    vscode.workspace.onDidCloseTextDocument(schedule),
    vscode.window.onDidChangeVisibleTextEditors(() => { void reconcile(); }),
    vscode.window.onDidChangeActiveTextEditor(updateUi)
  );
  const watcher = vscode.workspace.createFileSystemWatcher(`**/*.{${EXTENSIONS.join(',')}}`);
  context.subscriptions.push(watcher, watcher.onDidCreate(schedule), watcher.onDidChange(schedule), watcher.onDidDelete(schedule));
  const serverCommand = () => config().get<string>('serverPath') || context.asAbsolutePath(`bin/state-lens${process.platform === 'win32' ? '.exe' : ''}`);
  let sessionSubscriptions: vscode.Disposable[] = [];
  let session: { stopping: boolean } | undefined;
  const lifecycle = new ServerLifecycle({
    invalidate: () => {
      epoch++;
      requests.forEach(request => request.cancel());
      if (timer) clearTimeout(timer);
      clear();
    },
    changed: state => {
      status.text = `State Lens: ${config().get<boolean>('enabled', true) ? state : 'off'}`;
      updateUi();
      if (state === 'running') schedule();
    },
    error: error => {
      output.appendLine(`[Error] ${String(error)}`);
      status.text = 'State Lens: unavailable';
      updateUi();
    },
    start: async () => {
      sessionSubscriptions.forEach(subscription => subscription.dispose());
      sessionSubscriptions = [];
      if (client) await client.dispose();
      const command = serverCommand();
      output.appendLine(`Starting ${command} serve (extension ${context.extension.packageJSON.version})`);
      const startingSession = { stopping: false };
      session = startingSession;
      const sessionOutput: vscode.OutputChannel = { ...output, appendLine: value => {
        if (startingSession.stopping && value.includes(CLEAN_SERVER_EXIT)) return;
        output.appendLine(value);
      } };
      client = new LanguageClient(CONFIG_SECTION, 'State Lens', { command, args: ['serve'] }, {
        documentSelector: LANGUAGES.map(language => ({ scheme: 'file', language })),
        initializationOptions: { disabledChecks: config().get<string[]>('disabledChecks', []), diagnostics: config().get<boolean>('diagnostics', true) },
        middleware: { provideHover: () => null, provideInlayHints: () => [] },
        errorHandler: { error: () => ({ action: ErrorAction.Continue }), closed: () => ({ action: CloseAction.DoNotRestart }) },
        synchronize: { fileEvents: watcher, configurationSection: CONFIG_SECTION }, outputChannel: sessionOutput
      });
      sessionSubscriptions.push(client.onNotification('stateLens/indexing', schedule), client.onDidChangeState(event => { if (event.newState === State.Stopped && lifecycle.state === 'running') lifecycle.crashed(); }));
      await client.start();
      const capabilities = client.initializeResult?.capabilities;
      const supported: unknown = capabilities?.experimental?.stateLens?.languages;
      compatible = capabilities?.experimental?.stateLens?.stateAnnotations === 2 && divisionCompatible(capabilities) && Array.isArray(supported) && supported.length === 1 && supported[0] === LANGUAGE;
      if (!compatible) throw new Error(`Server ${command} is incompatible with State Lens (${LANGUAGE}). Reinstall the extension or correct ${CONFIG_SECTION}.serverPath.`);
    },
    stop: async () => {
      if (session) session.stopping = true;
      sessionSubscriptions.forEach(subscription => subscription.dispose());
      sessionSubscriptions = [];
      const retiring = client;
      client = undefined;
      if (retiring?.isRunning()) await retiring.dispose();
    }
  });
  async function reconcile(retry = false): Promise<void> {
    updateUi();
    await lifecycle.update(!disposed && config().get<boolean>('enabled', true) && vscode.window.visibleTextEditors.some(supports), retry);
    if (lifecycle.state === 'running') schedule();
  }
  shutdown = async () => {
    disposed = true;
    updateUi();
    await lifecycle.update(false);
  };
  status.text = 'State Lens: idle';
  await reconcile();
  return api;
}
export async function deactivate(): Promise<void> { await shutdown?.(); shutdown = undefined; }
