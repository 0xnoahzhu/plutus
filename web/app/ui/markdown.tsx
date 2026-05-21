/// Minimal markdown→JSX renderer. Built in-house instead of pulling in
/// `marked` or `remark` because the corpus is well-bounded (agent reports
/// + user-typed notes) and the dep budget is tight. Supports headings
/// (`#` → `######`), bold (`**`), italic (`*` / `_`), inline code
/// (`` ` ``), unordered lists (`-`, `*`, `+`), ordered lists (`1.` etc),
/// links (`[text](url)`), GFM-style pipe tables, and paragraphs separated
/// by blank lines.
///
/// Output goes through the JSX runtime, so all text is escaped — no
/// `innerHTML` / `dangerouslySetInnerHTML` paths. URLs in links are
/// further restricted to `http`, `https`, `mailto` to keep
/// `javascript:` out of agent-authored content.

import { css, type RemixNode } from 'remix/ui'

import { color, font, radius, shadow, space } from './tokens.ts'

/// Block wrapping `renderMarkdown` with a header that toggles between
/// the rendered preview and the raw source. The toggle is data-attribute
/// based — a tiny global click handler in `document.tsx` flips
/// `data-md-view` on the container. No re-render needed: both views are
/// in the DOM and CSS handles visibility.
///
/// `defaultView` is `'rendered'` so users see the formatted output by
/// default; the source is one click away.
export function MarkdownToggle() {
  return ({
    source,
    defaultView = 'rendered',
    labels = { rendered: 'Preview', raw: 'Source' },
  }: {
    source: string
    defaultView?: 'rendered' | 'raw'
    labels?: { rendered: string; raw: string }
  }) => {
    let rendered = renderMarkdown(source)
    return (
      <div
        data-md-view={defaultView}
        mix={css({
          margin: `${space[2]} 0 0`,
        })}
      >
        <div
          role="tablist"
          aria-label="View mode"
          mix={css({
            display: 'inline-flex',
            padding: '2px',
            marginBottom: space[2],
            background: color.bg,
            border: `1px solid ${color.borderSoft}`,
            borderRadius: radius.pill,
            gap: '2px',
          })}
        >
          <MdToggleButton target="rendered" label={labels.rendered} />
          <MdToggleButton target="raw" label={labels.raw} />
        </div>
        <div
          data-md-rendered=""
          mix={css({
            padding: `${space[3]} ${space[3]}`,
            background: color.surface,
            border: `1px solid ${color.borderSoft}`,
            borderRadius: radius.md,
            color: color.text,
          })}
        >
          {rendered}
        </div>
        <pre
          data-md-raw=""
          mix={css({
            margin: 0,
            padding: `${space[3]} ${space[3]}`,
            background: color.bg,
            border: `1px solid ${color.borderSoft}`,
            borderRadius: radius.md,
            fontSize: font.sm,
            lineHeight: 1.6,
            color: color.text,
            whiteSpace: 'pre-wrap',
            wordBreak: 'break-word',
            fontFamily: font.mono,
          })}
        >
          {source}
        </pre>
      </div>
    )
  }
}

function MdToggleButton() {
  return ({ target, label }: { target: 'rendered' | 'raw'; label: string }) => (
    <button
      type="button"
      role="tab"
      data-md-toggle=""
      data-md-target={target}
      // The initial `data-active` here matches the default-view button.
      // The click handler in document.tsx updates this on every toggle so
      // the soft-active pill follows the selected mode.
      data-active={target === 'rendered' ? 'true' : 'false'}
      mix={css({
        // Visual base.
        display: 'inline-flex',
        alignItems: 'center',
        padding: `${space[1]} ${space[3]}`,
        fontSize: font.xs,
        fontWeight: 600,
        border: 'none',
        cursor: 'pointer',
        borderRadius: radius.pill,
        // Inactive: ghosted text on transparent.
        background: 'transparent',
        color: color.textMuted,
        boxShadow: 'none',
        transition: 'background 120ms ease, color 120ms ease, box-shadow 120ms ease',
        // Active: white pill on the inset gray track. Driven entirely by
        // `[data-active="true"]` so the click handler doesn't need to
        // know about styling.
        '&[data-active="true"]': {
          background: color.surface,
          color: color.text,
          boxShadow: shadow.card,
        },
        '&:hover': { color: color.text },
      })}
    >
      {label}
    </button>
  )
}

/// Top-level render entry. Takes a markdown string, returns a list of
/// block-level JSX nodes ready to drop into a container.
export function renderMarkdown(source: string): RemixNode[] {
  let blocks = splitBlocks(source)
  return blocks.map((b, i) => renderBlock(b, i))
}

/// Split the source into block-level chunks. A block ends at a blank
/// line. List items in the same list are kept together — they only
/// break on an actual blank line OR a transition between list and
/// non-list, not just on a newline.
function splitBlocks(source: string): string[] {
  // Normalize line endings, drop trailing newlines.
  let lines = source.replace(/\r\n?/g, '\n').replace(/\n+$/, '').split('\n')
  let blocks: string[] = []
  let buf: string[] = []
  let inList = false

  let flush = () => {
    if (buf.length > 0) blocks.push(buf.join('\n'))
    buf = []
  }

  for (let raw of lines) {
    let line = raw
    let listLine = /^\s*(?:[-*+]|\d+\.)\s+/.test(line)
    let blank = line.trim() === ''

    if (blank) {
      flush()
      inList = false
      continue
    }
    // Heading is always its own block.
    if (/^#{1,6}\s/.test(line)) {
      flush()
      blocks.push(line)
      inList = false
      continue
    }
    // Transition between list and paragraph forces a flush.
    if (listLine !== inList && buf.length > 0) {
      flush()
    }
    inList = listLine
    buf.push(line)
  }
  flush()
  return blocks
}

/// Dispatch one block to the right renderer.
function renderBlock(block: string, key: number): RemixNode {
  let headingMatch = block.match(/^(#{1,6})\s+(.*)$/)
  if (headingMatch) {
    let level = headingMatch[1].length
    let text = headingMatch[2]
    return renderHeading(level, text, key)
  }
  let lines = block.split('\n')
  // GFM table: header row, separator row (`|---|:---:|---:|`), body
  // rows. The separator's existence is the structural marker — without
  // it, what looks like a table is just a paragraph with pipes.
  if (lines.length >= 2 && lines[0].includes('|') && isTableSeparator(lines[1])) {
    return renderTable(lines, key)
  }
  // Lists: every line in the block starts with a list marker. Use the
  // first line's marker to decide ordered vs unordered.
  let isList = lines.every((l) => /^\s*(?:[-*+]|\d+\.)\s+/.test(l))
  if (isList) {
    let ordered = /^\s*\d+\.\s+/.test(lines[0])
    return renderList(lines, ordered, key)
  }
  return renderParagraph(block, key)
}

function renderHeading(level: number, text: string, key: number): RemixNode {
  let sizes: Record<number, string> = {
    1: font.xl,
    2: font.lg,
    3: font.md,
    4: font.md,
    5: font.sm,
    6: font.sm,
  }
  let weights: Record<number, number> = { 1: 700, 2: 700, 3: 600, 4: 600, 5: 600, 6: 600 }
  let topGap: Record<number, string> = {
    1: space[4],
    2: space[4],
    3: space[3],
    4: space[3],
    5: space[2],
    6: space[2],
  }
  let style = css({
    margin: `${topGap[level]} 0 ${space[2]}`,
    fontSize: sizes[level],
    fontWeight: weights[level],
    color: color.text,
    lineHeight: 1.3,
    // Drop the top margin on the very first block so the card edge
    // doesn't get a giant gap before the first heading.
    '&:first-child': { marginTop: 0 },
  })
  let children = renderInline(text)
  switch (level) {
    case 1:
      return (
        <h1 key={key} mix={style}>
          {children}
        </h1>
      )
    case 2:
      return (
        <h2 key={key} mix={style}>
          {children}
        </h2>
      )
    case 3:
      return (
        <h3 key={key} mix={style}>
          {children}
        </h3>
      )
    case 4:
      return (
        <h4 key={key} mix={style}>
          {children}
        </h4>
      )
    case 5:
      return (
        <h5 key={key} mix={style}>
          {children}
        </h5>
      )
    default:
      return (
        <h6 key={key} mix={style}>
          {children}
        </h6>
      )
  }
}

function renderList(lines: string[], ordered: boolean, key: number): RemixNode {
  let items = lines.map((l, i) => {
    let text = l.replace(/^\s*(?:[-*+]|\d+\.)\s+/, '')
    return (
      <li key={i} mix={css({ margin: `${space[1]} 0`, lineHeight: 1.6 })}>
        {renderInline(text)}
      </li>
    )
  })
  let listStyle = css({
    margin: `${space[2]} 0`,
    paddingLeft: space[5],
    color: color.text,
    fontSize: font.sm,
  })
  return ordered ? (
    <ol key={key} mix={listStyle}>
      {items}
    </ol>
  ) : (
    <ul key={key} mix={listStyle}>
      {items}
    </ul>
  )
}

/// Detect a GFM table separator row: a line whose cells are all
/// dashes (optionally bookended by `:` for alignment), separated by
/// pipes. The leading and trailing pipes are optional in GFM.
function isTableSeparator(line: string): boolean {
  let trimmed = line.trim()
  if (!trimmed.includes('-')) return false
  let inner = trimmed.replace(/^\|/, '').replace(/\|$/, '')
  let cells = inner.split('|')
  if (cells.length === 0) return false
  return cells.every((c) => /^\s*:?-{1,}:?\s*$/.test(c))
}

type Align = 'left' | 'center' | 'right'

function parseAlign(separatorCell: string): Align {
  let c = separatorCell.trim()
  let left = c.startsWith(':')
  let right = c.endsWith(':')
  if (left && right) return 'center'
  if (right) return 'right'
  return 'left'
}

function splitTableRow(line: string): string[] {
  let trimmed = line.trim()
  let inner = trimmed.replace(/^\|/, '').replace(/\|$/, '')
  // Honor `\|` as a literal pipe inside a cell.
  let cells: string[] = []
  let buf = ''
  for (let i = 0; i < inner.length; i++) {
    let ch = inner[i]
    if (ch === '\\' && inner[i + 1] === '|') {
      buf += '|'
      i++
      continue
    }
    if (ch === '|') {
      cells.push(buf.trim())
      buf = ''
      continue
    }
    buf += ch
  }
  cells.push(buf.trim())
  return cells
}

function renderTable(lines: string[], key: number): RemixNode {
  let headers = splitTableRow(lines[0])
  let aligns = splitTableRow(lines[1]).map(parseAlign)
  let bodyRows = lines.slice(2).map(splitTableRow)
  let alignFor = (j: number): Align => aligns[j] ?? 'left'
  let cellBase = {
    padding: `${space[2]} ${space[3]}`,
    verticalAlign: 'top',
    borderBottom: `1px solid ${color.borderSoft}`,
  }
  // Outer wrapper scrolls horizontally on narrow viewports so wide
  // agent-emitted tables don't blow out the card width.
  return (
    <div
      key={key}
      mix={css({
        margin: `${space[3]} 0`,
        overflowX: 'auto',
        border: `1px solid ${color.borderSoft}`,
        borderRadius: radius.md,
      })}
    >
      <table
        mix={css({
          width: '100%',
          borderCollapse: 'collapse',
          fontSize: font.sm,
          color: color.text,
          fontVariantNumeric: 'tabular-nums',
        })}
      >
        <thead>
          <tr mix={css({ background: color.bg })}>
            {headers.map((h, j) => (
              <th
                key={j}
                mix={css({
                  ...cellBase,
                  textAlign: alignFor(j),
                  fontWeight: 600,
                  fontSize: font.xs,
                  textTransform: 'uppercase',
                  letterSpacing: '0.05em',
                  color: color.textMuted,
                  whiteSpace: 'nowrap',
                })}
              >
                {renderInline(h)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {bodyRows.map((row, i) => (
            <tr
              key={i}
              mix={css({
                '&:last-child td': { borderBottom: 'none' },
              })}
            >
              {headers.map((_, j) => (
                <td
                  key={j}
                  mix={css({
                    ...cellBase,
                    textAlign: alignFor(j),
                  })}
                >
                  {renderInline(row[j] ?? '')}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

function renderParagraph(block: string, key: number): RemixNode {
  // Join soft-wrapped lines with a space; the user's prose flows.
  let joined = block.split('\n').join(' ')
  return (
    <p
      key={key}
      mix={css({
        margin: `${space[2]} 0`,
        fontSize: font.sm,
        lineHeight: 1.65,
        color: color.text,
        '&:first-child': { marginTop: 0 },
        '&:last-child': { marginBottom: 0 },
      })}
    >
      {renderInline(joined)}
    </p>
  )
}

/// Tokenize a single line/paragraph for inline markup. Order matters:
/// inline code first (its contents are literal), then links, then
/// `**bold**`, then `*italic*` / `_italic_`. Anything that doesn't match
/// falls through as plain text.
function renderInline(text: string): RemixNode[] {
  let out: RemixNode[] = []
  let i = 0
  let buf = ''

  let flush = () => {
    if (buf.length > 0) {
      out.push(buf)
      buf = ''
    }
  }

  while (i < text.length) {
    let ch = text[i]

    // Inline code: `...`
    if (ch === '`') {
      let end = text.indexOf('`', i + 1)
      if (end > i) {
        flush()
        let body = text.slice(i + 1, end)
        out.push(
          <code
            key={out.length}
            mix={css({
              fontFamily: font.mono,
              fontSize: '0.9em',
              padding: '1px 5px',
              borderRadius: radius.sm,
              background: color.bg,
              border: `1px solid ${color.borderSoft}`,
              color: color.text,
            })}
          >
            {body}
          </code>,
        )
        i = end + 1
        continue
      }
    }

    // Link: [text](url)
    if (ch === '[') {
      let closeBracket = findUnescaped(text, ']', i + 1)
      if (closeBracket > i && text[closeBracket + 1] === '(') {
        let closeParen = findUnescaped(text, ')', closeBracket + 2)
        if (closeParen > closeBracket + 1) {
          let label = text.slice(i + 1, closeBracket)
          let url = text.slice(closeBracket + 2, closeParen).trim()
          if (isSafeUrl(url)) {
            flush()
            out.push(
              <a
                key={out.length}
                href={url}
                rel="nofollow noopener noreferrer"
                target="_blank"
                mix={css({
                  color: color.brand,
                  textDecoration: 'underline',
                  textUnderlineOffset: '2px',
                  '&:hover': { color: color.brandHover },
                })}
              >
                {renderInline(label)}
              </a>,
            )
            i = closeParen + 1
            continue
          }
        }
      }
    }

    // Bold: **...**
    if (ch === '*' && text[i + 1] === '*') {
      let end = text.indexOf('**', i + 2)
      if (end > i + 1) {
        flush()
        let body = text.slice(i + 2, end)
        out.push(
          <strong key={out.length} mix={css({ fontWeight: 700, color: color.text })}>
            {renderInline(body)}
          </strong>,
        )
        i = end + 2
        continue
      }
    }

    // Italic: *...*  or  _..._
    if ((ch === '*' || ch === '_') && text[i + 1] !== ch) {
      let end = text.indexOf(ch, i + 1)
      // Avoid matching the second half of `**` we already handled above.
      if (end > i && text[end + 1] !== ch && text[end - 1] !== ' ') {
        flush()
        let body = text.slice(i + 1, end)
        out.push(
          <em key={out.length} mix={css({ fontStyle: 'italic' })}>
            {renderInline(body)}
          </em>,
        )
        i = end + 1
        continue
      }
    }

    buf += ch
    i++
  }
  flush()
  return out
}

function findUnescaped(text: string, needle: string, from: number): number {
  let i = from
  while (i < text.length) {
    if (text[i] === '\\' && i + 1 < text.length) {
      i += 2
      continue
    }
    if (text[i] === needle) return i
    i++
  }
  return -1
}

function isSafeUrl(url: string): boolean {
  // Allow relative paths and our three known protocols. Anything else
  // (`javascript:`, `data:`, etc.) gets stripped — link renders as plain
  // text instead. The agent shouldn't be emitting non-http URLs anyway.
  if (url.startsWith('/') || url.startsWith('#')) return true
  return /^(https?:|mailto:)/i.test(url)
}
