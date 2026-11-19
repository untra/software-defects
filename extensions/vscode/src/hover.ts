import { commandId } from './package';
import * as vscode from 'vscode';
import { PALETTE, Category, Entry } from './ranges';

const SITE_ORIGIN = 'https://badsoftware.untra.io';
const SITE_LOCALE = 'en';
export const EVIDENCE_COMMAND = commandId('showEvidence');
const OBJECT_IDENTITY = 'object-identity';
const SHORT_EXPLANATIONS: Record<string, string> = {
  'Mutable binding; local mutation does not imply observable impurity.': 'Mutable binding.'
};

function defectGuideUrl(family: string, defect: string, compoundClass?: string): string {
  return `${SITE_ORIGIN}/${SITE_LOCALE}/${encodeURIComponent(family)}/${compoundClass ? encodeURIComponent(compoundClass) + "/" : ""}${encodeURIComponent(defect)}/`;
}
function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/g, character => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[character]!));
}
export function explanation(entry: Entry, uri?: string, version?: number): vscode.MarkdownString {
  const text = new vscode.MarkdownString();
  const key = entry.finding.compound?.class ?? entry.finding.family;
  const color = Object.prototype.hasOwnProperty.call(PALETTE, key) ? PALETTE[key as Category] : undefined;
  text.supportHtml = true;
  if (color) text.appendMarkdown(`<span style="color:${color};"><strong>${escapeHtml(entry.finding.defect)}</strong></span>`);
  else text.appendText(entry.finding.defect);
  text.appendText(`\n\n${SHORT_EXPLANATIONS[entry.finding.explanation] ?? entry.finding.explanation}`);
  const shared = entry.finding.shared_state;
  if (shared) {
    const sites = shared.sites;
    text.appendText(`\n\nShared-state evidence: ${shared.evidence_level}.`);
    text.appendText(`\nStatic sites: ${sites.declarations} async declarations, ${sites.launches} launches/compositions, ${sites.suspensions} suspensions.`);
    if (sites.dynamic_fan_out) text.appendText(' Runtime fan-out is unknown.');
    text.appendText(`\nRelated execution scopes: ${shared.contenders}. Events: ${shared.events.map(event => event.kind).join(' → ')}.`);
    if (shared.assumptions.length) text.appendText(`\nLimits: ${shared.assumptions.join(' ')}`);
  }
  const trace = entry.finding.state_trace;
  if (trace) {
    text.appendText(`\n\nEvidence: ${trace.evidence_level.replaceAll('_', ' ')}`);
    if (trace.required_condition) text.appendText(`\nRequired condition: ${trace.required_condition}`);
    text.appendText(`\nOrigin locations: ${trace.origins.length}; invalidations: ${trace.invalidations.length}; affected uses: ${trace.affected_uses.length}.`);
    if (entry.finding.resolution === OBJECT_IDENTITY) {
      text.appendText(`\nMechanism: ${entry.finding.mechanism ?? 'unknown'}. `);
      text.appendText(trace.requirement
        ? `Declared requirement (${trace.requirement.kind}) is violated by this write.`
        : 'No declared isolation requirement: a state relationship, not a confirmed defect.');
    }
    if (trace.assumptions.length) text.appendText(`\nAssumptions: ${trace.assumptions.join(' ')}`);
  }
  text.appendMarkdown(`\n\n[Guide](<${defectGuideUrl(entry.finding.family, entry.finding.defect, entry.finding.compound?.class)}>)`);
  if (uri !== undefined && version !== undefined) {
    const args = encodeURIComponent(JSON.stringify([uri, version, entry.finding]));
    text.appendMarkdown(` · [Evidence](command:${EVIDENCE_COMMAND}?${args})`);
    text.isTrusted = { enabledCommands: [EVIDENCE_COMMAND] };
  }
  return text;
}
