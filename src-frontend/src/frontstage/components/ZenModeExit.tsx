interface ZenModeExitProps {
  onExit: () => void;
}

export default function ZenModeExit({ onExit }: ZenModeExitProps) {
  return (
    <button onClick={onExit} className="zen-mode-exit">
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
      >
        <path d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3" />
      </svg>
      退出禅模式 (F11)
    </button>
  );
}
