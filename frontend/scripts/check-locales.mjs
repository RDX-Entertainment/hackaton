import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';

const file = new URL('../src/i18n/textCatalog.ts', import.meta.url);
const source = ts.createSourceFile('textCatalog.ts', readFileSync(file, 'utf8'), ts.ScriptTarget.Latest, true);
const entries = source.statements.find(ts.isVariableStatement).declarationList.declarations[0].initializer.text;
const seen = new Map();
const placeholders = (text) => [...text.matchAll(/\{\d+\}/g)].map(([key]) => key).sort();
for (const line of entries.trim().split('\n')) {
  const [key, en, pt, es = key, ...extra] = line.split('|');
  assert.equal(extra.length, 0, `Unexpected separator: ${key}`);
  assert.ok(key && en && pt && es, `Missing translation: ${key}`);
  for (const text of [en, pt, es]) assert.deepEqual(placeholders(text), placeholders(key), `Parameters: ${key}`);
  if (seen.has(key)) assert.equal(line, seen.get(key), `Conflicting duplicate: ${key}`);
  seen.set(key, line);
}
console.log(`${seen.size} interface messages: ES/EN/PT present, placeholders consistent.`);
