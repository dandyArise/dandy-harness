document.addEventListener('DOMContentLoaded', () => {
  const style = document.createElement('style');
  style.textContent = `
    [data-model-menu]:has([id$="-models"]) {
      width: min(450px, calc(100vw - 24px)); max-width: min(450px, calc(100vw - 24px));
      max-height: min(510px, calc(100dvh - 100px)); padding: 8px; gap: 4px;
    }
    [data-model-menu] input[placeholder] {
      box-sizing: border-box; min-height: 32px; padding-inline: 10px;
      border-radius: 7px; background: color-mix(in srgb,currentColor 6%,transparent);
    }
    [data-model-menu] label:has(input[type="checkbox"]) {
      display: inline-flex; align-items: center; gap: 5px; padding: 3px 8px;
      border: 1px solid var(--dsw-alias-border-l1); border-radius: 6px; font-size: 12px;
    }
    [data-model-menu] [data-menu-group] [data-menu-group-heading] {
      position: static; margin: 8px 0 2px; padding: 4px 9px; border-radius: 0;
      background: transparent; color: var(--dsw-alias-label-secondary);
      font-size: 11px; font-weight: 650; letter-spacing: .035em;
    }
    [data-model-menu] [data-menu-group] + [data-menu-group] {
      border-top: 1px solid var(--dsw-alias-border-l1);
    }
    [data-model-menu] [data-menu-group] button[role="menuitemradio"] {
      padding: 8px 10px; border-radius: 7px;
    }
    [data-model-menu] [data-menu-group] button[aria-checked="true"] {
      background: color-mix(in srgb,currentColor 8%,transparent);
    }
    [data-model-menu] button small { display: flex; flex-wrap: wrap; gap: 3px 5px; margin-top: 4px; }
    [data-model-menu] button small > span[aria-label] {
      padding: 1px 5px; border-radius: 4px; background: color-mix(in srgb,currentColor 7%,transparent);
      font-size: 11px; line-height: 16px; white-space: nowrap;
    }
    [data-model-menu] button small > span:not([aria-label]):last-child {
      flex-basis: 100%; font-size: 11px; line-height: 17px; font-variant-numeric: tabular-nums;
    }
  `;
  document.head.append(style);
});
