// Fills every .glyph with Claude Code's pixel creature.
const rows = [
  '.#......#.',
  '##########',
  '##e####e##',
  '##########',
  '.########.',
  '.##....##.',
  '.#......#.',
];
for (const g of document.querySelectorAll('.glyph')) {
  g.innerHTML = rows.flatMap((r) => [...r].map((c) => `<i class="${c === '#' ? 'p' : c === 'e' ? 'e' : ''}"></i>`)).join('');
}
