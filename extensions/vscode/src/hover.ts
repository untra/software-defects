import * as vscode from 'vscode';
import type { Entry } from './ranges';

const SITE_ORIGIN = 'https://badsoftware.untra.io';
const SITE_LOCALE = 'en';

function defectGuideUrl(family: string, defect: string): string {
  return `${SITE_ORIGIN}/${SITE_LOCALE}/${encodeURIComponent(family)}/${encodeURIComponent(defect)}/`;
}

export function explanation(entry: Entry): vscode.MarkdownString {
  const text = new vscode.MarkdownString();
  text.appendText(`${entry.finding.defect}: ${entry.finding.explanation}\n\n`);
  text.appendText(`${entry.finding.origin} • ${entry.finding.effect.kind} • ${entry.finding.resolution}\n\nSusceptibility is not a bug diagnosis.`);
  text.appendMarkdown(`\n\n[Read the defect guide ↗](<${defectGuideUrl(entry.finding.family, entry.finding.defect)}>)`);
  if (entry.finding.related_cwes?.length) text.appendText(`\n\nRelated taxonomy: ${entry.finding.related_cwes.join(', ')} (not diagnosed).`);
  for (const evidence of [...entry.finding.call_path, ...entry.finding.evidence]) {
    text.appendText(`\n${vscode.Uri.parse(evidence.uri).fsPath} [bytes ${evidence.range.start}–${evidence.range.end}]`);
  }
  return text;
}
