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

/// Page-wide submit guard. Two responsibilities:
///
/// 1. **Confirm modal** — any submit button with a non-empty `title`
///    attribute prompts via our styled dialog before the request goes
///    out. Replaces the browser-native `confirm()` so it matches the UI.
///
/// 2. **Fetch-based POST submission** — instead of letting the browser
///    natively navigate on POST submit, we intercept and use `fetch()`
///    with `redirect: 'follow'`, then `location.replace(resp.url)`.
///    This means the browser's navigation history NEVER contains a POST
///    entry, so refresh on the destination page can never trigger
///    "Confirm Form Resubmission" — regardless of browser quirks.
///
/// Why intercept rather than rely on PRG alone: every controller already
/// returns 303 redirects (POST-Redirect-Get), which is the spec-correct
/// way to avoid resubmit warnings. But some browsers (older Safari in
/// particular) still warn on F5 in edge cases after PRG. Going through
/// fetch eliminates the problem at the source: the browser never sees
/// the POST as a navigation, so there's nothing to "resubmit".
///
/// Flow for a destructive submit (button has title):
///   1. submit event fires → preventDefault → modal opens
///   2. User confirms → `submitViaFetch(form, submitter)` runs
///   3. Set-Cookie / 303 / final GET all happen inside fetch
///   4. `location.replace(resp.url)` lands the user on the destination
///      with no POST in history
///
/// Flow for a non-destructive submit (no title):
///   1. submit event fires → preventDefault → straight to submitViaFetch
///   2. Same fetch + replace pattern
///
/// Falls back gracefully if JS is disabled: the form submits natively
/// (standard HTML behavior). Our controllers all return 303, so the
/// no-JS path still works — it's just subject to the original
/// browser-quirk warning the interceptor was added to dodge.
const CONFIRM_SUBMIT_JS = `
  (function() {
    var modal = document.getElementById('confirm-modal');
    if (!modal) return;
    var promptEl = document.getElementById('confirm-modal-prompt');
    var cancelBtn = document.getElementById('confirm-modal-cancel');
    var confirmBtn = document.getElementById('confirm-modal-confirm');
    var pendingForm = null;
    var pendingSubmitter = null;

    function openModal(text) {
      promptEl.textContent = text;
      modal.setAttribute('data-open', 'true');
      // Defer focus until the next paint so the transition can start.
      setTimeout(function() { cancelBtn.focus(); }, 0);
    }
    function closeModal() {
      modal.removeAttribute('data-open');
      pendingForm = null;
      pendingSubmitter = null;
    }

    // Submit a form via fetch, then navigate to the final URL.
    //
    // The fetch follows the 303 from the POST handler; resp.url is the
    // destination GET. Calling location.replace(resp.url) leaves the
    // browser's history with only that GET — no POST entry, no
    // "Confirm Form Resubmission" warning on F5.
    //
    // The fetch-follow does GET resp.url to retrieve the response body
    // (which we then discard), and location.replace fires a second GET
    // to actually navigate. That double-GET means anything one-shot on
    // the server side gets hit twice — keep that in mind when adding
    // GET handlers that mutate request-scoped state.
    function submitViaFetch(form, submitter) {
      var data = new FormData(form);
      // Match native semantics: include the clicked submitter's name/value
      // if it has a name. Forms can rely on knowing which button fired.
      if (submitter && submitter.name) {
        data.append(submitter.name, submitter.value || '');
      }
      var action = form.getAttribute('action') || window.location.href;
      fetch(action, {
        method: 'POST',
        body: data,
        credentials: 'same-origin',
        redirect: 'follow',
      }).then(function(resp) {
        if (resp.redirected || resp.url !== action) {
          window.location.replace(resp.url);
        } else {
          // Server returned the same URL without redirecting — rare,
          // but reload so we pick up whatever state changed.
          window.location.reload();
        }
      }).catch(function(err) {
        console.error('form submit failed:', err);
        // Network error — reload so the user sees a fresh state.
        window.location.reload();
      });
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
      var s = pendingSubmitter;
      closeModal();
      // Confirmed destructive action — go through fetch like any other
      // submit so the destination history entry is a clean GET.
      if (f) submitViaFetch(f, s);
    });

    document.addEventListener('submit', function(e) {
      var form = e.target;
      if (!form || (form.method || 'GET').toUpperCase() !== 'POST') return;
      var btn = e.submitter;
      // Always intercept POSTs — even non-destructive ones — so the
      // browser never tracks the POST in its navigation history.
      e.preventDefault();
      var prompt = btn && btn.type === 'submit' && btn.getAttribute('title');
      if (prompt) {
        pendingForm = form;
        pendingSubmitter = btn;
        openModal(prompt);
      } else {
        submitViaFetch(form, btn);
      }
    });

    // Copy-to-clipboard click handler. Any element carrying a
    // [data-copy="<text>"] attribute is treated as a copy trigger;
    // clicking copies the attribute's value and briefly swaps the
    // button's label to a "Copied!" string. Used by the freshly-minted
    // API token banner so the user can grab the secret with one click.
    //
    // Prefers the Clipboard API (HTTPS-only in modern Chrome) and
    // falls back to the textarea + execCommand('copy') trick when the
    // page is on plain HTTP.
    document.addEventListener('click', function(e) {
      var btn = e.target && e.target.closest && e.target.closest('[data-copy]');
      if (!btn) return;
      e.preventDefault();
      var text = btn.getAttribute('data-copy');
      var doneLabel = btn.getAttribute('data-copy-done') || 'Copied!';
      var originalText = btn.textContent;

      function flash() {
        btn.textContent = doneLabel;
        setTimeout(function() { btn.textContent = originalText; }, 1500);
      }
      function legacyCopy() {
        var ta = document.createElement('textarea');
        ta.value = text;
        ta.setAttribute('readonly', '');
        ta.style.position = 'fixed';
        ta.style.left = '-9999px';
        document.body.appendChild(ta);
        ta.select();
        try { document.execCommand('copy'); flash(); }
        catch (err) { console.error('copy failed:', err); }
        finally { document.body.removeChild(ta); }
      }

      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(flash, legacyCopy);
      } else {
        legacyCopy();
      }
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
