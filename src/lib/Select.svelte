<script lang="ts" generics="T extends string | number | null">
  import { onDestroy, tick } from "svelte";

  type Option = { value: T; label: string; hint?: string };

  let {
    value = $bindable(),
    options,
    onchange,
    disabled = false,
    placeholder = "Select…",
    size = "md",
  }: {
    value: T;
    options: Option[];
    onchange?: (v: T) => void;
    disabled?: boolean;
    placeholder?: string;
    size?: "md" | "sm";
  } = $props();

  let open = $state(false);
  let trigger: HTMLButtonElement | undefined = $state();
  let panel: HTMLDivElement | undefined = $state();
  let panelStyle = $state("");
  let activeIndex = $state(-1);

  const current = $derived(options.find((o) => o.value === value));
  const label = $derived(current?.label ?? placeholder);

  async function place() {
    if (!trigger) return;
    const r = trigger.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    const width = r.width;
    const spaceBelow = vh - r.bottom - 8;
    const spaceAbove = r.top - 8;
    const maxHeight = Math.min(240, Math.max(spaceBelow, spaceAbove));
    const openUp = spaceBelow < 120 && spaceAbove > spaceBelow;
    const top = openUp
      ? Math.max(4, r.top - Math.min(maxHeight, spaceAbove) - 4)
      : r.bottom + 4;
    const left = Math.min(Math.max(4, r.left), vw - width - 4);
    panelStyle = `top: ${top}px; left: ${left}px; width: ${width}px; max-height: ${maxHeight}px;`;
  }

  async function openMenu() {
    if (disabled) return;
    open = true;
    activeIndex = Math.max(0, options.findIndex((o) => o.value === value));
    await tick();
    await place();
    panel?.focus();
    scrollActiveIntoView();
  }

  function closeMenu() {
    open = false;
    activeIndex = -1;
  }

  function scrollActiveIntoView() {
    if (!panel || activeIndex < 0) return;
    const el = panel.querySelector<HTMLElement>(
      `[data-idx="${activeIndex}"]`,
    );
    el?.scrollIntoView({ block: "nearest" });
  }

  function pick(v: T) {
    value = v;
    onchange?.(v);
    closeMenu();
    trigger?.focus();
  }

  function onKey(e: KeyboardEvent) {
    if (!open) {
      if (e.key === "ArrowDown" || e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        openMenu();
      }
      return;
    }
    if (e.key === "Escape") {
      e.preventDefault();
      closeMenu();
      trigger?.focus();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      activeIndex = Math.min(options.length - 1, activeIndex + 1);
      scrollActiveIntoView();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      activeIndex = Math.max(0, activeIndex - 1);
      scrollActiveIntoView();
    } else if (e.key === "Home") {
      e.preventDefault();
      activeIndex = 0;
      scrollActiveIntoView();
    } else if (e.key === "End") {
      e.preventDefault();
      activeIndex = options.length - 1;
      scrollActiveIntoView();
    } else if (e.key === "Enter") {
      e.preventDefault();
      const opt = options[activeIndex];
      if (opt) pick(opt.value);
    }
  }

  function onDocClick(e: MouseEvent) {
    if (!open) return;
    const t = e.target as Node;
    if (trigger?.contains(t)) return;
    if (panel?.contains(t)) return;
    closeMenu();
  }

  function onReflow() {
    if (open) place();
  }

  $effect(() => {
    if (open) {
      document.addEventListener("mousedown", onDocClick, true);
      window.addEventListener("scroll", onReflow, true);
      window.addEventListener("resize", onReflow);
      return () => {
        document.removeEventListener("mousedown", onDocClick, true);
        window.removeEventListener("scroll", onReflow, true);
        window.removeEventListener("resize", onReflow);
      };
    }
  });

  onDestroy(() => closeMenu());
</script>

<button
  bind:this={trigger}
  type="button"
  class="trigger size-{size}"
  class:open
  class:placeholder={!current}
  {disabled}
  aria-haspopup="listbox"
  aria-expanded={open}
  onclick={() => (open ? closeMenu() : openMenu())}
  onkeydown={onKey}
>
  <span class="lbl">{label}</span>
  <svg class="chev" width="10" height="10" viewBox="0 0 12 12" aria-hidden="true">
    <path
      d="M2.5 4.5 L6 8 L9.5 4.5"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
</button>

{#if open}
  <div
    bind:this={panel}
    class="panel"
    role="listbox"
    tabindex="-1"
    style={panelStyle}
    onkeydown={onKey}
  >
    {#each options as opt, i (String(opt.value))}
      <button
        type="button"
        data-idx={i}
        class="opt"
        class:active={i === activeIndex}
        class:selected={opt.value === value}
        role="option"
        aria-selected={opt.value === value}
        onmouseenter={() => (activeIndex = i)}
        onclick={() => pick(opt.value)}
      >
        <span class="opt-label">{opt.label}</span>
        {#if opt.value === value}
          <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
            <path
              d="M2.5 6.5 L5 9 L9.5 3.5"
              fill="none"
              stroke="currentColor"
              stroke-width="1.6"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        {/if}
      </button>
    {/each}
  </div>
{/if}

<style>
  .trigger {
    display: inline-flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    width: 100%;
    background: var(--surface);
    color: var(--ink);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    padding: 11px 12px 11px 14px;
    font-family: inherit;
    font-size: 14px;
    line-height: 1.2;
    cursor: pointer;
    transition:
      border-color 0.14s ease,
      background 0.14s ease;
    text-align: left;
  }
  .trigger:hover:not(:disabled) {
    border-color: var(--line-strong);
  }
  .trigger.open {
    border-color: var(--accent);
    box-shadow: var(--focus-ring);
  }
  .trigger:disabled {
    background: var(--sunk);
    color: var(--ink-muted);
    cursor: not-allowed;
  }
  .trigger.placeholder {
    color: var(--ink-muted);
  }
  .trigger.size-sm {
    padding: 5px 8px 5px 10px;
    font-size: 12px;
    background: var(--sunk);
    border-color: transparent;
    border-radius: 6px;
  }
  .trigger.size-sm:hover:not(:disabled) {
    background: #ebe8e0;
    border-color: transparent;
  }
  .trigger.size-sm.open {
    background: var(--surface);
    border-color: var(--accent);
  }

  .lbl {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .chev {
    color: var(--ink-muted);
    flex-shrink: 0;
  }

  .panel {
    position: fixed;
    z-index: 100;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    padding: 4px;
    box-shadow:
      0 8px 24px rgba(27, 27, 31, 0.12),
      0 2px 6px rgba(27, 27, 31, 0.06),
      0 0 0 0.5px rgba(27, 27, 31, 0.04);
    overflow-y: auto;
    outline: none;
    display: flex;
    flex-direction: column;
    gap: 1px;
    animation: panelIn 0.11s ease-out;
  }

  @keyframes panelIn {
    from {
      opacity: 0;
      transform: translateY(-2px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .panel { animation: none; }
  }

  .opt {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    background: transparent;
    color: var(--ink);
    border: none;
    border-radius: 6px;
    padding: 7px 10px;
    font-family: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
    transition: background 0.08s ease;
  }
  .opt.active {
    background: var(--sunk);
  }
  .opt.selected {
    color: var(--accent);
  }
  .opt-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
