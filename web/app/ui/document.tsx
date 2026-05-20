import type { RemixNode } from 'remix/ui'

import { messages } from '../i18n/messages.ts'
import { routes } from '../routes.ts'

import { color, font, radius, shadow, space, THEME_CSS } from './tokens.ts'

export interface DocumentProps {
  children?: RemixNode
  title?: string
  /// BCP-47 language tag for the document. Defaults to "en".
  lang?: string
  /// Resolved color-scheme choice: `system` lets CSS `prefers-color-scheme`
  /// decide; `dark` / `light` force the palette via a `data-theme` attribute.
  /// Defaults to `system`.
  theme?: 'system' | 'dark' | 'light'
}

const DEFAULT_TITLE = 'Plutus'

/// Tiny global stylesheet. Lives inline so we don't have to wire up a static
/// CSS file or PostCSS pipeline yet — fine for the single-user app size.
/// `body` colors are token-driven so the theme switch swaps them in CSS.
///
/// The `.confirm-modal*` rules drive the custom destructive-action dialog
/// rendered at the bottom of `<body>`. Hidden by default; the submit-guard
/// script flips `data-open="true"` on `#confirm-modal` to show it.
const GLOBAL_CSS = `
  *, *::before, *::after { box-sizing: border-box; }
  html, body {
    margin: 0;
    padding: 0;
    background: ${color.bg};
    color: ${color.text};
    font-family: ${font.sans};
    font-size: ${font.base};
    line-height: 1.5;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    text-rendering: optimizeLegibility;
  }
  a { color: inherit; }
  button { font-family: inherit; }
  table { border-collapse: collapse; }

  .confirm-modal-root {
    position: fixed;
    inset: 0;
    z-index: 9999;
    display: none;
    align-items: center;
    justify-content: center;
    padding: ${space[5]};
    background: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(2px);
    -webkit-backdrop-filter: blur(2px);
    /* Fade in: opacity transitions when [data-open] toggles. */
    opacity: 0;
    transition: opacity 120ms ease;
  }
  .confirm-modal-root[data-open="true"] {
    display: flex;
    opacity: 1;
  }
  .confirm-modal-card {
    width: 100%;
    max-width: 440px;
    background: ${color.surface};
    border: 1px solid ${color.border};
    border-radius: ${radius.lg};
    box-shadow: ${shadow.card};
    padding: ${space[5]} ${space[5]} ${space[4]};
    display: flex;
    flex-direction: column;
    gap: ${space[3]};
  }
  .confirm-modal-title {
    margin: 0;
    font-size: ${font.xs};
    font-weight: 600;
    color: ${color.textMuted};
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .confirm-modal-prompt {
    margin: 0;
    font-size: ${font.base};
    color: ${color.text};
    line-height: 1.55;
  }
  .confirm-modal-actions {
    margin-top: ${space[2]};
    display: flex;
    justify-content: flex-end;
    gap: ${space[2]};
    flex-wrap: wrap;
  }
  .confirm-modal-btn {
    padding: ${space[2]} ${space[4]};
    border-radius: ${radius.md};
    font-size: ${font.base};
    font-weight: 600;
    cursor: pointer;
    border: 1px solid ${color.border};
    background: ${color.surface};
    color: ${color.text};
    transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
  }
  .confirm-modal-btn:hover { background: ${color.hover}; }
  .confirm-modal-btn-danger {
    background: ${color.danger};
    border-color: ${color.danger};
    color: #fff;
  }
  .confirm-modal-btn-danger:hover {
    background: ${color.dangerText};
    border-color: ${color.dangerText};
  }
`

/// Page-wide submit guard with a custom modal. Any submit button that
/// carries a non-empty `title` attribute triggers our styled confirm
/// dialog instead of the browser-native `window.confirm()` (which looks
/// out of place on top of the themed UI).
///
/// Flow:
///   1. Submit event fires; we read `event.submitter` to find the
///      specific button the user clicked (so multiple destructive
///      buttons inside one form prompt with the right copy).
///   2. If the button has a non-empty `title`, we `preventDefault` the
///      submit and capture the form + button.
///   3. The modal pops with the prompt as body text. Cancel / Escape /
///      backdrop click all close it without doing anything. Confirm
///      calls `form.submit()` on the captured form — that bypasses our
///      listener entirely (programmatic submits don't fire `submit`
///      events) so the request goes through cleanly.
///   4. Focus lands on the Cancel button by default — Enter cancels
///      rather than confirms, which is safer for destructive actions.
const CONFIRM_SUBMIT_JS = `
  (function() {
    var modal = document.getElementById('confirm-modal');
    if (!modal) return;
    var promptEl = document.getElementById('confirm-modal-prompt');
    var cancelBtn = document.getElementById('confirm-modal-cancel');
    var confirmBtn = document.getElementById('confirm-modal-confirm');
    var pendingForm = null;

    function openModal(text) {
      promptEl.textContent = text;
      modal.setAttribute('data-open', 'true');
      // Defer focus until the next paint so the transition can start.
      setTimeout(function() { cancelBtn.focus(); }, 0);
    }
    function closeModal() {
      modal.removeAttribute('data-open');
      pendingForm = null;
    }
    cancelBtn.addEventListener('click', closeModal);
    modal.addEventListener('click', function(e) {
      // Only the backdrop dismisses; clicks inside the card don't.
      if (e.target === modal) closeModal();
    });
    document.addEventListener('keydown', function(e) {
      if (e.key === 'Escape' && modal.getAttribute('data-open') === 'true') {
        closeModal();
      }
    });
    confirmBtn.addEventListener('click', function() {
      var f = pendingForm;
      closeModal();
      // form.submit() bypasses the submit-event listener entirely, so
      // the request goes through without re-prompting.
      if (f) f.submit();
    });

    document.addEventListener('submit', function(e) {
      var btn = e.submitter;
      if (!btn || btn.type !== 'submit') return;
      var prompt = btn.getAttribute('title');
      if (!prompt) return;
      e.preventDefault();
      pendingForm = e.target;
      openModal(prompt);
    });
  })();
`

export function Document() {
  return ({ title = DEFAULT_TITLE, lang = 'en', theme = 'system', children }: DocumentProps) => {
    // `data-theme="dark"|"light"` pins the palette; `system` omits the attr
    // and lets the `prefers-color-scheme` rule in THEME_CSS decide.
    let themeAttr: Record<string, string> =
      theme === 'system' ? {} : { 'data-theme': theme }
    let confirm = messages(lang).confirms
    return (
      <html lang={lang} {...themeAttr}>
        <head>
          <meta charSet="utf-8" />
          <meta name="viewport" content="width=device-width, initial-scale=1" />
          <meta name="color-scheme" content="light dark" />
          <title>{title}</title>
          {/* Inter as a progressive enhancement. system-ui fallback below. */}
          <link rel="preconnect" href="https://fonts.googleapis.com" />
          <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
          <link
            rel="stylesheet"
            href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap"
          />
          <style innerHTML={THEME_CSS} />
          <style innerHTML={GLOBAL_CSS} />
        </head>
        <body>
          {children}
          {/* Shared destructive-action confirm dialog. Hidden by default;
              the submit-guard script flips `data-open` on the root. */}
          <div
            id="confirm-modal"
            class="confirm-modal-root"
            role="dialog"
            aria-modal="true"
            aria-labelledby="confirm-modal-title"
            aria-describedby="confirm-modal-prompt"
          >
            <div class="confirm-modal-card">
              <p id="confirm-modal-title" class="confirm-modal-title">
                {confirm.dialogTitle}
              </p>
              <p id="confirm-modal-prompt" class="confirm-modal-prompt" />
              <div class="confirm-modal-actions">
                <button
                  type="button"
                  id="confirm-modal-cancel"
                  class="confirm-modal-btn"
                >
                  {confirm.cancelLabel}
                </button>
                <button
                  type="button"
                  id="confirm-modal-confirm"
                  class="confirm-modal-btn confirm-modal-btn-danger"
                >
                  {confirm.confirmLabel}
                </button>
              </div>
            </div>
          </div>
          <script type="module" src={routes.assets.href({ path: 'app/assets/entry.ts' })}></script>
          <script innerHTML={CONFIRM_SUBMIT_JS}></script>
        </body>
      </html>
    )
  }
}
