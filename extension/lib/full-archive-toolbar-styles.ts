export function toolbarStyles(): string {
  return `<style>
    :host { all: initial; }
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

    :host {
      --toolbar-bg: rgba(255,255,255,0.94);
      --toolbar-stroke: rgba(0,0,0,0.10);
      --text-primary: #1D1D1F;
      --text-secondary: #6E6E73;
      --text-tertiary: #AEAEB2;
      --text-quaternary: #C7C7CC;
      --text-on-color: #FFFFFF;
      --border-secondary: rgba(0,0,0,0.12);
      --fill-hover: rgba(0,0,0,0.04);
      --fill-subtle: rgba(0,0,0,0.035);
      --accent: #0071E3;
      --accent-soft: rgba(0,113,227,0.10);
      --success: #34C759;
      --success-soft: rgba(52,199,89,0.12);
      --warning: #FF9500;
      --warning-soft: rgba(255,149,0,0.13);
      --destructive: #FF3B30;
      --destructive-soft: rgba(255,59,48,0.10);
      --bg-elevated: #FFFFFF;
    }
    @media (prefers-color-scheme: dark) {
      :host {
        --toolbar-bg: rgba(28,28,30,0.94);
        --toolbar-stroke: rgba(255,255,255,0.13);
        --text-primary: #F5F5F7;
        --text-secondary: #B4B4BA;
        --text-tertiary: #636366;
        --text-quaternary: #48484A;
        --text-on-color: #FFFFFF;
        --border-secondary: rgba(255,255,255,0.14);
        --fill-hover: rgba(255,255,255,0.07);
        --fill-subtle: rgba(255,255,255,0.05);
        --accent: #0A84FF;
        --accent-soft: rgba(10,132,255,0.16);
        --success: #30D158;
        --success-soft: rgba(48,209,88,0.15);
        --warning: #FF9F0A;
        --warning-soft: rgba(255,159,10,0.17);
        --destructive: #FF453A;
        --destructive-soft: rgba(255,69,58,0.14);
        --bg-elevated: #2C2C2E;
      }
    }

    .bar {
      position: fixed; top: 0; left: 0; right: 0; z-index: 2147483647;
      height: 44px; display: flex; align-items: center; gap: 12px; padding: 0 14px;
      background: var(--toolbar-bg);
      border-bottom: 0.5px solid var(--toolbar-stroke);
      backdrop-filter: blur(20px) saturate(180%);
      -webkit-backdrop-filter: blur(20px) saturate(180%);
      transform: translateY(-110%); transition: transform 180ms ease;
      font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", sans-serif;
      -webkit-font-smoothing: antialiased;
    }
    .bar.is-open { transform: translateY(0); }

    .brand-mark {
      width: 20px; height: 20px; display: block; flex-shrink: 0;
      box-shadow: 0 0.5px 2px rgba(0,0,0,0.28);
    }
    .left-lockup { display: inline-flex; align-items: center; gap: 8px; flex-shrink: 0; }
    .item { flex: 1; min-width: 0; display: flex; align-items: center; gap: 7px; }
    .item .t {
      font-size: 13px; font-weight: 600; letter-spacing: -0.01em;
      color: var(--text-primary); white-space: nowrap; overflow: hidden;
      text-overflow: ellipsis; max-width: 380px;
    }
    .item .s {
      font-size: 12px; font-weight: 400; letter-spacing: -0.005em;
      color: var(--text-secondary); white-space: nowrap; overflow: hidden;
      text-overflow: ellipsis; flex: 1;
    }
    .item .dot { color: var(--text-quaternary); font-size: 12px; flex-shrink: 0; }
    .item-name { font-size: 13px; font-weight: 600; letter-spacing: -0.01em; color: var(--text-primary); white-space: nowrap; }
    .group { display: inline-flex; align-items: center; gap: 2px; flex-shrink: 0; }

    .ic { width: 15px; height: 15px; display: block; flex-shrink: 0; stroke: currentColor; stroke-width: 1.4; stroke-linecap: round; stroke-linejoin: round; fill: none; }
    .ic.filled { fill: currentColor; stroke: none; }

    button { cursor: pointer; border: none; background: transparent; font-family: inherit; -webkit-font-smoothing: antialiased; }
    a { text-decoration: none; font-family: inherit; }

    .btn-text {
      height: 26px; padding: 0 6px 0 8px; border-radius: 6px;
      display: inline-flex; align-items: center; gap: 3px;
      font-size: 13px; font-weight: 600; letter-spacing: -0.01em;
      color: var(--text-primary); transition: background 120ms ease;
    }
    .btn-text:hover { background: var(--fill-hover); }
    .btn-text .ic { color: var(--text-tertiary); }

    .ic-btn {
      width: 26px; height: 26px; border-radius: 6px;
      display: inline-flex; align-items: center; justify-content: center;
      color: var(--text-secondary); transition: background 120ms ease, color 120ms ease;
    }
    .ic-btn:hover { background: var(--fill-hover); color: var(--text-primary); }
    .ic-btn.panel-open { color: var(--accent); background: var(--accent-soft); }
    .ic-btn.starred .ic { fill: #F3A800; stroke: #F3A800; }

    .dropdown {
      height: 26px; padding: 0 4px 0 8px; border-radius: 6px;
      display: inline-flex; align-items: center; gap: 5px;
      font-size: 13px; font-weight: 500; letter-spacing: -0.01em;
      color: var(--text-primary); transition: background 120ms ease;
    }
    .dropdown:hover, .dropdown.triage-open { background: var(--fill-hover); }
    .dropdown .ic { color: var(--text-secondary); }

    .count-pill {
      height: 20px; min-width: 24px; padding: 0 8px; border-radius: 980px;
      background: var(--fill-subtle); border: 0.5px solid var(--border-secondary);
      color: var(--text-secondary); font-size: 11px; font-weight: 600; letter-spacing: -0.005em;
      display: inline-flex; align-items: center; justify-content: center;
    }

    .toggle-group {
      display: inline-flex; align-items: center; gap: 6px; padding: 0 4px;
      color: var(--text-secondary); font-size: 12px; font-weight: 500; letter-spacing: -0.005em;
    }
    .switch {
      width: 28px; height: 16px; border-radius: 980px; background: var(--accent);
      position: relative; flex-shrink: 0; transition: background 150ms ease; cursor: pointer;
    }
    .switch.off { background: rgba(120,120,128,0.30); }
    .switch .knob {
      position: absolute; top: 1.5px; left: 13.5px;
      width: 13px; height: 13px; border-radius: 50%; background: #FFFFFF;
      box-shadow: 0 1px 2px rgba(0,0,0,0.25), 0 0 0 0.5px rgba(0,0,0,0.04);
      transition: left 150ms ease; pointer-events: none;
    }
    .switch.off .knob { left: 1.5px; }

    .vr { width: 0.5px; height: 20px; background: var(--border-secondary); margin: 0 4px; flex-shrink: 0; }

    .btn-primary {
      height: 26px; padding: 0 12px; border-radius: 6px;
      background: var(--accent); color: var(--text-on-color);
      display: inline-flex; align-items: center; gap: 6px;
      font-size: 12px; font-weight: 600; letter-spacing: -0.005em;
      transition: filter 120ms ease;
    }
    .btn-primary:hover { filter: brightness(1.08); }

    .btn-ghost {
      height: 26px; padding: 0 10px; border-radius: 6px;
      background: transparent; color: var(--text-primary);
      border: 0.5px solid var(--border-secondary);
      display: inline-flex; align-items: center; gap: 6px;
      font-size: 12px; font-weight: 500; letter-spacing: -0.005em;
      transition: background 120ms ease;
    }
    .btn-ghost:hover { background: var(--fill-hover); }

    .url-input {
      height: 26px; padding: 0 10px; border-radius: 6px; width: 130px;
      background: transparent; color: var(--text-primary);
      border: 0.5px solid var(--border-secondary);
      font: 500 12px/26px system-ui; letter-spacing: -0.005em;
      outline: none; transition: background 120ms ease, border-color 120ms ease;
    }
    .url-input:hover { background: var(--fill-hover); }
    .url-input:focus { border-color: var(--accent); background: transparent; }

    .status {
      height: 22px; padding: 0 10px; border-radius: 980px;
      display: inline-flex; align-items: center; gap: 6px;
      font-size: 11px; font-weight: 600; letter-spacing: 0.005em;
    }
    .status.saving { background: var(--warning-soft); color: var(--warning); }
    .status.checking { background: var(--accent-soft); color: var(--accent); }

    .spinner {
      width: 9px; height: 9px; border-radius: 50%;
      border: 1.4px solid currentColor; border-right-color: transparent;
      animation: spin 800ms linear infinite; flex-shrink: 0;
    }
    @keyframes spin { to { transform: rotate(360deg); } }

    /* Sub-panels */
    .trs-panel {
      position: fixed; top: 44px; left: 0; right: 0; z-index: 2147483646;
      background: var(--toolbar-bg); border-bottom: 0.5px solid var(--toolbar-stroke);
      backdrop-filter: blur(20px) saturate(180%); -webkit-backdrop-filter: blur(20px) saturate(180%);
      padding: 12px 16px 14px;
      animation: panel-in 160ms cubic-bezier(.2,.85,.2,1) both;
      font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", sans-serif;
      -webkit-font-smoothing: antialiased;
    }
    @keyframes panel-in { from { opacity:0; transform:translateY(-6px); } to { opacity:1; transform:translateY(0); } }

    .panel-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; }
    .panel-title { font-size: 12px; font-weight: 600; letter-spacing: -0.005em; color: var(--text-primary); }

    .tag-chips { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 10px; }
    .tag-chip {
      height: 24px; padding: 0 6px 0 10px; border-radius: 980px;
      background: var(--fill-subtle); border: 0.5px solid var(--border-secondary);
      color: var(--text-primary); font-size: 12px; font-weight: 500;
      display: inline-flex; align-items: center; gap: 4px;
      font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", sans-serif;
    }
    .tag-remove {
      width: 16px; height: 16px; border-radius: 50%; border: none; background: transparent;
      color: var(--text-tertiary); font-size: 14px; line-height: 1; cursor: pointer;
      display: inline-flex; align-items: center; justify-content: center; margin-left: 2px;
      transition: color 100ms ease, background 100ms ease;
    }
    .tag-remove:hover { color: var(--text-primary); background: var(--fill-hover); }
    .tag-input-wrap { display: flex; align-items: center; gap: 8px; }
    .tag-input {
      flex: 1; max-width: 280px; height: 28px; padding: 0 10px; border-radius: 7px;
      border: 0.5px solid var(--border-secondary); background: var(--bg-elevated);
      color: var(--text-primary); font-size: 13px; letter-spacing: -0.01em; outline: none;
      font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", sans-serif;
      transition: border-color 120ms ease, box-shadow 120ms ease;
    }
    .tag-input::placeholder { color: var(--text-tertiary); }
    .tag-input:focus { border-color: var(--accent); box-shadow: 0 0 0 2.5px var(--accent-soft); }

    .note-textarea {
      display: block; width: 100%; height: 80px; padding: 8px 10px; border-radius: 8px;
      border: 0.5px solid var(--border-secondary); background: var(--bg-elevated);
      color: var(--text-primary); font-size: 13px; letter-spacing: -0.01em; line-height: 1.5;
      resize: none; outline: none; margin-bottom: 10px;
      font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", sans-serif;
      -webkit-font-smoothing: antialiased;
      transition: border-color 120ms ease, box-shadow 120ms ease;
    }
    .note-textarea::placeholder { color: var(--text-tertiary); }
    .note-textarea:focus { border-color: var(--accent); box-shadow: 0 0 0 2.5px var(--accent-soft); }
    .note-actions { display: flex; justify-content: flex-end; gap: 8px; }

    /* Triage dropdown */
    .triage-menu {
      position: fixed; z-index: 2147483648; min-width: 152px; padding: 4px;
      border-radius: 12px; background: var(--bg-elevated);
      border: 0.5px solid var(--border-secondary);
      box-shadow: 0 8px 32px rgba(0,0,0,0.14), 0 0 0 0.5px rgba(0,0,0,0.06);
      animation: menu-in 140ms cubic-bezier(.2,.85,.2,1) both;
      font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", sans-serif;
      -webkit-font-smoothing: antialiased;
    }
    @media (prefers-color-scheme: dark) {
      .triage-menu { box-shadow: 0 8px 32px rgba(0,0,0,0.60), 0 0 0 0.5px rgba(255,255,255,0.10); }
    }
    @keyframes menu-in { from { opacity:0; transform:scale(0.94) translateY(-4px); } to { opacity:1; transform:scale(1) translateY(0); } }

    .triage-item {
      display: flex; align-items: center; gap: 8px; width: 100%; padding: 7px 10px;
      border-radius: 8px; background: transparent; color: var(--text-primary);
      font-size: 13px; font-weight: 500; letter-spacing: -0.01em; text-align: left;
      transition: background 100ms ease; cursor: pointer;
    }
    .triage-item:hover { background: var(--fill-hover); }
    .triage-item.active { color: var(--accent); font-weight: 600; }
    .ti-ic { width: 14px; height: 14px; display: block; flex-shrink: 0; stroke: currentColor; stroke-width: 1.4; stroke-linecap: round; stroke-linejoin: round; fill: none; }
    .triage-item .ti-check { margin-left: auto; width: 14px; height: 14px; display: block; stroke: var(--accent); stroke-width: 1.8; stroke-linecap: round; stroke-linejoin: round; fill: none; }
    .triage-item:not(.active) .ti-check { display: none; }
    .triage-item .spacer { margin-left: auto; width: 14px; }
  </style>`
}
