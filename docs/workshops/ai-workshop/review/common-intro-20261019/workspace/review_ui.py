"""Review-page controls and text diffs; slide sources are never modified here."""
import difflib
import html
import json
import re
from html.parser import HTMLParser
from pathlib import Path


class NoteText(HTMLParser):
    def __init__(self, title):
        super().__init__(convert_charrefs=True)
        self.values = {'heading': title, 'lede': '', 'narration': ''}
        self.field = 'narration'
        self.label = None
        self.block_depth = 0

    def handle_starttag(self, tag, attrs):
        if tag == 'h4':
            self.label = ''
        elif tag in ('p', 'li', 'h3'):
            self.block_depth += 1
        elif tag == 'br':
            self.values[self.field] += '\n'

    def handle_endtag(self, tag):
        if tag == 'h4':
            field = {'見出し': 'heading', '導入文': 'lede', '原稿': 'narration'}.get(self.label)
            if field:
                self.field = field
                self.values[field] = ''
            self.label = None
        elif tag in ('p', 'li', 'h3') and self.label is None:
            self.values[self.field] += '\n\n'
            self.block_depth = max(0, self.block_depth - 1)

    def handle_data(self, data):
        if self.label is not None:
            self.label += data
        elif not self.block_depth and not data.strip():
            return
        else:
            self.values[self.field] += data


def fields(note):
    parser = NoteText(note.get('title', ''))
    parser.feed(note.get('body', ''))
    return {key: value.strip() for key, value in parser.values.items()}


def field_diff(before, after):
    chunks = []
    old_chunks, new_chunks = [], []
    added = removed = 0
    # Japanese sentence boundaries avoid matching isolated characters across unrelated sentences.
    old = re.findall(r'[^。！？\n]*[。！？\n]|[^。！？\n]+$', before)
    new = re.findall(r'[^。！？\n]*[。！？\n]|[^。！？\n]+$', after)
    assert ''.join(old) == before and ''.join(new) == after
    for tag, a, b, c, d in difflib.SequenceMatcher(None, old, new, autojunk=False).get_opcodes():
        old_text, new_text = ''.join(old[a:b]), ''.join(new[c:d])
        if tag == 'equal':
            escaped = html.escape(old_text)
            chunks.append(escaped)
            old_chunks.append(escaped)
            new_chunks.append(escaped)
        else:
            if a != b:
                removed += len(old_text)
                escaped = '<del>' + html.escape(old_text) + '</del>'
                chunks.append(escaped)
                old_chunks.append(escaped)
            if c != d:
                added += len(new_text)
                escaped = '<ins>' + html.escape(new_text) + '</ins>'
                chunks.append(escaped)
                new_chunks.append(escaped)
    return {'html': ''.join(chunks), 'before_html': ''.join(old_chunks), 'after_html': ''.join(new_chunks), 'changed': before != after, 'added': added, 'removed': removed}


def enhance(text):
    here = Path(__file__).resolve().parent
    match = re.search(r'<script type="application/json" id="data">(.*?)</script>', text, re.S)
    data = json.loads(match[1])
    diffs = {}
    for deck in data:
        rows = []
        old = deck.get('beforeNotes')
        for i, note in enumerate(deck.get('afterNotes', deck['notes'])):
            if old is None or i >= len(old):
                rows.append(None)
            else:
                before, after = fields(old[i]), fields(note)
                rows.append({key: field_diff(before[key], after[key]) for key in before})
        diffs[deck['id']] = rows

    start = text.index('</style>') + len('</style>')
    end = text.index('<script type="application/json" id="data">')
    body = text[start:end]
    references = body[body.index('<article id="design-references">'):body.index('<span id="lead">')]
    context = body[body.index('<article id="lead-review-notice">'):body.index('<div class="toolbar"><select id="deck"')]
    references = re.sub(r'<iframe\b', '<iframe loading="lazy"', references)
    context = re.sub(r'<iframe\b', '<iframe loading="lazy"', context)
    # Keep historical content available without placing it ahead of the current task.
    context = context.replace('id="lead-review-notice"', 'id="lead-review-notice" class="context-card"')
    text = text[:start] + (here / 'review-ui-shell.html').read_text().replace('<!-- REVIEW_CONTEXT -->', context).replace('<!-- REFERENCES -->', references) + text[end:]
    text = text.replace('</style>', '\n' + (here / 'review-ui.css').read_text() + '</style>', 1)
    encoded = json.dumps(diffs, ensure_ascii=False).replace('</', '<\\/')
    text = text.replace('<script type="application/json" id="data">', '<script type="application/json" id="review-diffs">' + encoded + '</script><script type="application/json" id="data">', 1)
    # Replace only the original comparison controller; preserve the reference-page scripts.
    text, replaced = re.subn(r'<script>\s*const data=JSON.parse\(document.querySelector\(\'#data\'\).*?</script>', lambda _: '<script>' + (here / 'review-ui.js').read_text() + '</script>', text, count=1, flags=re.S)
    assert replaced == 1, 'comparison controller not found'
    return text
