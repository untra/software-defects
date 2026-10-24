import * as vscode from 'vscode';
import { explanation } from './hover';
import { LanguageClient, LSPErrorCodes, ResponseError, State } from 'vscode-languageclient/node';
import { accepts, Entry, Evidence, segments, triggerSegments, Snapshot, Range } from './ranges';

const LANGUAGES = ['go', 'rust', 'typescript'];
const REFRESH_DELAY_MS = 180;
const GREEN = '120, 166, 90';
const CONFIG_SECTION = 'stateLens';
const SERVER_ENV = 'STATE_LENS_SERVER';
const SERVER_BINARY = 'state-lens';
let client: LanguageClient | undefined;

export interface Applied { version: number; findings: number; segments: number; triggers: number }
export interface AuditSnapshot extends Applied { audit: Snapshot }
export interface StateLensApi {
  log(): string[];
  snapshot(uri: string): Promise<AuditSnapshot | undefined>;
  serverRunning(): boolean;
  status(): string;
  applied(uri: string): Applied | undefined;
}

function range(value: Range): vscode.Range { return new vscode.Range(value.start.line, value.start.character, value.end.line, value.end.character); }
type EvidenceNode = vscode.TreeItem & { children?: vscode.TreeItem[] };
class EvidenceView implements vscode.TreeDataProvider<vscode.TreeItem> {
  private emitter = new vscode.EventEmitter<void>();
  readonly onDidChangeTreeData = this.emitter.event;
  private snapshots = new Map<string, Snapshot>();
  update(snapshot: Snapshot): void { this.snapshots.set(snapshot.uri, snapshot); this.emitter.fire(); }
  clear(): void { this.snapshots.clear(); this.emitter.fire(); }
  getTreeItem(item: vscode.TreeItem): vscode.TreeItem { return item; }
  getChildren(element?: EvidenceNode): vscode.TreeItem[] {
    if (element) return element.children ?? [];
    return [...this.snapshots.values()].flatMap(snapshot => snapshot.findings.map(entry => {
      const item: EvidenceNode = new vscode.TreeItem(`${entry.finding.origin}: ${entry.finding.defect}`, vscode.TreeItemCollapsibleState.Collapsed);
      item.children = [...entry.finding.call_path, ...entry.finding.evidence].map(evidence => {
        const child = new vscode.TreeItem(`${vscode.workspace.asRelativePath(evidence.uri)}: bytes ${evidence.range.start}–${evidence.range.end}`);
        child.command = { command: 'stateLens.openEvidence', title: 'Open originating evidence', arguments: [evidence] };
        return child;
      });
      item.description = `${vscode.workspace.asRelativePath(snapshot.uri)}:${entry.triggerRange.start.line + 1}`;
      item.tooltip = explanation(entry);
      item.command = { command: 'stateLens.showEvidence', title: 'Show evidence', arguments: [snapshot.uri, entry] };
      return item;
    }));
  }
  dispose(): void { this.emitter.dispose(); }
}

export async function activate(context: vscode.ExtensionContext): Promise<StateLensApi> {
  const evidence = new EvidenceView();
  context.subscriptions.push(evidence, vscode.window.registerTreeDataProvider('stateLens.evidence', evidence));
  const channel = vscode.window.createOutputChannel('State Lens');
  const recorded: string[] = [];
  let partial = '';
  const record = (value: string): void => {
    const lines = (partial + value).split('\n');
    partial = lines.pop() ?? '';
    recorded.push(...lines);
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
  let decorations: vscode.TextEditorDecorationType[] = [];
  const applied = new Map<string, Applied>();
  const api: StateLensApi = {
    log: () => [...recorded, ...(partial ? [partial] : [])],
    snapshot: async uri => {
      const document = vscode.workspace.textDocuments.find(doc => doc.uri.toString() === uri);
      if (!document || !client?.isRunning()) throw new Error('State Lens document or server unavailable');
      const requestedEpoch = epoch;
      const version = document.version;
      try {
        const snapshot = await client.sendRequest<Snapshot>('stateLens/ranges', { textDocument: { uri } });
        if (disposed || !accepts(snapshot, uri, document.version, requestedEpoch, epoch) || version !== document.version) return undefined;
        const directOnly = config().get<boolean>('directOnly', false);
        return { version, findings: snapshot.findings.length, segments: segments(snapshot, directOnly).length,
          triggers: triggerSegments(snapshot, directOnly).length, audit: snapshot };
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
    decorations = [opacity, opacity / 2, Math.min(opacity * 2, 0.5)].map(alpha => vscode.window.createTextEditorDecorationType({
      backgroundColor: `rgba(${GREEN}, ${alpha})`,
      border: config().get<boolean>('accessibleMarkers') ? '1px solid' : undefined,
      borderColor: new vscode.ThemeColor('focusBorder'),
      rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
      isWholeLine: false,
      overviewRulerColor: `rgba(${GREEN}, 0.6)`
    }));
  }
  function clear(): void {
    for (const editor of vscode.window.visibleTextEditors) decorations.forEach(d => editor.setDecorations(d, []));
    applied.clear();
    evidence.clear();
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
        const snapshot = await client.sendRequest<Snapshot>('stateLens/ranges', { textDocument: { uri } });
        if (disposed || !accepts(snapshot, uri, editor.document.version, requestedEpoch, epoch)) continue;
        const directOnly = config().get<boolean>('directOnly', false);
        const parts = segments(snapshot, directOnly);
        for (let i = 0; i < 2; i++) {
          editor.setDecorations(decorations[i], parts.filter(p => Number(p.propagated) === i).map(p => ({ range: range(p.range), hoverMessage: p.entries.map(explanation) })));
        }
        const triggers = triggerSegments(snapshot, directOnly);
        editor.setDecorations(decorations[2], triggers.map(part => ({ range: range(part.range), hoverMessage: explanation(part.entry) })));
        applied.set(uri, { version: editor.document.version, findings: snapshot.findings.length, segments: parts.length, triggers: triggers.length });
        evidence.update({ ...snapshot, findings: snapshot.findings.filter(e => !directOnly || e.finding.origin === 'direct') });
        incomplete ||= Boolean(snapshot.indexing) || snapshot.parseErrors || snapshot.unresolved.length > 0;
      }
      if (requestedEpoch === epoch) status.text = `State Lens: green${incomplete ? ' (partial)' : ''}`;
    } catch (error) {
      if (error instanceof ResponseError && error.code === LSPErrorCodes.ContentModified) { rerun = true; return; }
      clear();
      status.text = 'State Lens: unavailable';
      output.appendLine(`[Error] ${String(error)}`);
    } finally {
      running = false;
      if (rerun || epoch !== requestedEpoch) { rerun = false; schedule(); }
    }
  }
  styles();
  context.subscriptions.push({ dispose: () => { disposed = true; if (timer) clearTimeout(timer); decorations.forEach(d => d.dispose()); } });
  context.subscriptions.push(
    vscode.commands.registerCommand('stateLens.toggle', async () => { await config().update('enabled', !config().get<boolean>('enabled', true), vscode.ConfigurationTarget.Global); }),
    vscode.commands.registerCommand('stateLens.refresh', schedule),
    vscode.commands.registerCommand('stateLens.openEvidence', async (evidence: Evidence) => {
      const doc = await vscode.workspace.openTextDocument(vscode.Uri.parse(evidence.uri));
      const bytes = Buffer.from(doc.getText(), 'utf8');
      const start = doc.positionAt(bytes.subarray(0, evidence.range.start).toString('utf8').length);
      const end = doc.positionAt(bytes.subarray(0, evidence.range.end).toString('utf8').length);
      await vscode.window.showTextDocument(doc, { selection: new vscode.Range(start, end) });
    }),
    vscode.commands.registerCommand('stateLens.showEvidence', async (uri: string, entry: Entry) => {
      const doc = await vscode.workspace.openTextDocument(vscode.Uri.parse(uri));
      await vscode.window.showTextDocument(doc, { selection: range(entry.triggerRange) });
    }),
    vscode.workspace.onDidChangeConfiguration(event => { if (event.affectsConfiguration(CONFIG_SECTION)) { styles(); schedule(); } }),
    vscode.workspace.onDidChangeTextDocument(event => { if (LANGUAGES.includes(event.document.languageId)) schedule(); }),
    vscode.workspace.onDidCloseTextDocument(schedule),
    vscode.window.onDidChangeVisibleTextEditors(schedule)
  );
  const watcher = vscode.workspace.createFileSystemWatcher('**/*.{go,rs,ts}');
  context.subscriptions.push(watcher, watcher.onDidCreate(schedule), watcher.onDidChange(schedule), watcher.onDidDelete(schedule));
  const command = config().get<string>('serverPath') || process.env[SERVER_ENV] || SERVER_BINARY;
  output.appendLine(`Starting ${command} serve`);
  client = new LanguageClient(CONFIG_SECTION, 'State Lens', { command, args: ['serve'] }, {
    documentSelector: LANGUAGES.map(language => ({ scheme: 'file', language })),
    initializationOptions: { diagnostics: config().get<boolean>('diagnostics', false) },
    synchronize: { fileEvents: watcher, configurationSection: CONFIG_SECTION }, outputChannel: output
  });
  context.subscriptions.push(client.onDidChangeState(event => { if (event.newState === State.Stopped) { epoch++; clear(); status.text = 'State Lens: stopped'; } }));
  context.subscriptions.push(client.onNotification('stateLens/indexing', () => schedule()));
  try { await client.start(); status.show(); schedule(); }
  catch (error) { output.appendLine(`[Error] ${String(error)}`); void vscode.window.showErrorMessage('State Lens could not start. Install the state-lens binary or configure stateLens.serverPath.'); }
  return api;
}
export async function deactivate(): Promise<void> { await client?.stop(); client = undefined; }
