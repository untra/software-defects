import { commandId } from './package';
import * as vscode from 'vscode';
import { PALETTE, Category, Entry } from './ranges';

const SITE_ORIGIN = 'https://badsoftware.untra.io';
const SITE_LOCALE = 'en';
export const EVIDENCE_COMMAND = commandId('showEvidence');
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
  const trace = entry.finding.state_trace;
  if (trace) {
    text.appendText(`\n\nEvidence: ${trace.evidence_level.replaceAll('_', ' ')}`);
    if (trace.required_condition) text.appendText(`\nRequired condition: ${trace.required_condition}`);
    text.appendText(`\nOrigin locations: ${trace.origins.length}; invalidations: ${trace.invalidations.length}; affected uses: ${trace.affected_uses.length}.`);
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
