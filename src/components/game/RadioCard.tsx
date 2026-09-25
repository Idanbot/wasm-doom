export function RadioCard({ speaker, text }: { speaker: string; text: string }) {
  return (
    <aside className="radio-card" aria-live="polite">
      <span>{speaker}</span>
      <p>{text}</p>
    </aside>
  );
}
