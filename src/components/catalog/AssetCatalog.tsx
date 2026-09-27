import { useEffect, useMemo, useRef, useState } from "react";
import { Check, Copy, Pause, Play, Search, Volume2 } from "lucide-react";
import { asset } from "@/lib/asset";
import { WEAPONS } from "@/components/game/data";
import catalogDataRaw from "@/lib/asset-catalog-data.json";
import draftWeaponsRaw from "@/lib/draft-weapons-v2-data.json";
import draftAssetsRaw from "@/lib/draft-assets-v2-data.json";

type AssetItem = {
  file: string;
  name: string;
  size: number;
  hash: string;
  shortHash: string;
  width?: number;
  height?: number;
  group?: string;
  seamless?: boolean;
};
type DraftWeapon = {
  id: string;
  slot: number;
  is_boss: boolean;
  sheet_file: string;
  sheet_hash: string;
  sheet_short_hash: string;
  aim_file: string;
  aim_hash: string;
  aim_short_hash: string;
  gen_master_file?: string;
  gen_master_hash?: string;
  gen_master_short_hash?: string;
  cell_w: number;
  cell_h: number;
  frames_total: number;
};

const drafts = (draftWeaponsRaw as DraftWeapon[]).sort((a, b) => a.slot - b.slot);
const allAssets = [...new Map(
  [...(catalogDataRaw as AssetItem[]), ...(draftAssetsRaw as AssetItem[])]
    .filter((item) => !item.file.startsWith("/game/draft/") || item.file.startsWith("/game/draft/v2/"))
    .map((item) => [item.file, item] as const),
).values()];
const modes = [
  { name: "Full aim", start: 0, count: 1 },
  { name: "Half ammo", start: 1, count: 1 },
  { name: "Low ammo", start: 2, count: 1 },
  { name: "Empty mag", start: 3, count: 1 },
  { name: "No ammo", start: 4, count: 1 },
  { name: "Dry fire", start: 5, count: 1 },
  { name: "Pickup", start: 6, count: 4 },
  { name: "Reload", start: 10, count: 5 },
  { name: "Fire", start: 15, count: 5 },
  { name: "Special", start: 20, count: 5 },
] as const;
type Tab = "weapons" | "items" | "enemies" | "particles" | "textures" | "audio" | "all";
const tabs: Tab[] = ["weapons", "items", "enemies", "particles", "textures", "audio", "all"];

function groupOf(item: AssetItem): Tab {
  if (item.file.includes("/music/") || item.file.includes("/sfx/") || item.file.includes("/voices/")) return "audio";
  if (item.name.startsWith("enemy_")) return "enemies";
  if (item.group === "textures" || /^(wall_|floor_|ceil_|tex_)/.test(item.name)) return "textures";
  if (item.group === "particles" || /^(spr_flame|spr_muzzle|spr_impact|spr_ball|projectile_)/.test(item.name)) return "particles";
  if (item.group === "items" || /^(spr_gun_|spr_med|spr_armor|spr_ammo|spr_ord|spr_crate|spr_barrel|spr_lamp|spr_chain)/.test(item.name)) return "items";
  return "all";
}

export function AssetCatalog({ onBackToGame }: { onBackToGame?: () => void }) {
  const [tab, setTab] = useState<Tab>("weapons");
  const [selected, setSelected] = useState(0);
  const [modeIndex, setModeIndex] = useState(0);
  const [frame, setFrame] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [search, setSearch] = useState("");
  const [copied, setCopied] = useState("");
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const draft = drafts[selected] ?? drafts[0]!;
  const weapon = WEAPONS[draft.slot]!;
  const mode = modes[modeIndex]!;
  const cell = mode.start + Math.min(frame, mode.count - 1);
  const filtered = useMemo(() => allAssets.filter((item) => {
    if (tab !== "all" && tab !== "weapons" && groupOf(item) !== tab) return false;
    const q = search.trim().toLowerCase();
    return !q || `${item.name} ${item.file} ${item.hash}`.toLowerCase().includes(q);
  }), [tab, search]);

  useEffect(() => {
    if (!playing || mode.count === 1 || tab !== "weapons") return;
    const timer = window.setInterval(() => setFrame((value) => (value + 1) % mode.count), 160);
    return () => window.clearInterval(timer);
  }, [playing, mode.count, tab]);

  const copy = (value: string) => {
    void navigator.clipboard.writeText(value);
    setCopied(value);
  };
  const hashButton = (hash: string, short: string) => (
    <button type="button" onClick={() => copy(hash)} className="flex items-center gap-1 rounded border border-slate-700 px-2 py-1 text-xs text-amber-300 hover:border-amber-500" title="Copy full SHA-256">
      {copied === hash ? <Check size={13} /> : <Copy size={13} />} #{short}
    </button>
  );
  const selectMode = (index: number) => { setModeIndex(index); setFrame(0); setPlaying(false); };

  return <div className="min-h-screen bg-[#080d11] text-slate-200">
    <header className="sticky top-0 z-20 flex flex-wrap items-center justify-between gap-3 border-b border-amber-900/50 bg-[#0b1116]/95 px-4 py-3 backdrop-blur">
      <div><h1 className="font-mono text-sm font-bold tracking-[.18em] text-amber-400">BLACKSITE / ASSET CATALOG</h1><p className="mt-1 font-mono text-xs text-slate-400">DEV ONLY · LATEST 5×5 WEAPON SET · {drafts.length} GUNS</p></div>
      <div className="flex items-center gap-3">
        <label className="flex items-center gap-2 rounded border border-slate-700 bg-black/40 px-2"><Search size={16} /><input aria-label="Search assets" value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Search assets or hash" className="w-40 bg-transparent py-2 text-xs outline-none sm:w-64" /></label>
        {onBackToGame && <button type="button" onClick={onBackToGame} className="rounded bg-amber-500 px-3 py-2 text-xs font-bold text-black">BACK TO GAME</button>}
      </div>
    </header>
    <nav aria-label="Asset categories" className="flex gap-1 overflow-x-auto border-b border-slate-800 px-4 py-2 font-mono text-xs">
      {tabs.map((entry) => <button type="button" key={entry} onClick={() => setTab(entry)} className={`shrink-0 rounded px-3 py-2 uppercase ${tab === entry ? "bg-amber-500/20 text-amber-300" : "text-slate-400 hover:bg-slate-800"}`}>{entry === "weapons" ? `Weapons (${drafts.length})` : entry}</button>)}
    </nav>
    <main className="mx-auto max-w-[1500px] p-4">
      {tab === "weapons" ? <div className="grid gap-5 lg:grid-cols-[290px_minmax(0,1fr)]">
        <aside className="max-h-[72vh] space-y-2 overflow-y-auto rounded border border-slate-800 bg-[#101820] p-3">
          {drafts.map((entry, index) => {
            const spec = WEAPONS[entry.slot];
            if (!spec || (search && !`${spec.name} ${entry.id}`.toLowerCase().includes(search.toLowerCase()))) return null;
            return <button type="button" key={entry.id} onClick={() => { setSelected(index); setFrame(0); }} className={`flex w-full items-center gap-3 rounded border p-2 text-left ${selected === index ? "border-amber-500 bg-amber-500/10" : "border-slate-800 bg-black/20 hover:border-slate-600"}`}>
              <img src={asset(entry.aim_file)} alt="" className="h-12 w-16 object-contain" />
              <span className="min-w-0"><strong className="block truncate text-xs">{entry.slot + 1}. {spec.name}</strong><small className="text-[10px] text-slate-400">{entry.is_boss ? "BOSS REWARD" : "SECTOR WEAPON"} · #{entry.sheet_short_hash}</small></span>
            </button>;
          })}
        </aside>
        <section className="min-w-0 space-y-4">
          <div className="rounded border border-slate-800 bg-[#101820] p-4">
            <div className="mb-3 flex flex-wrap items-start justify-between gap-2"><div><h2 className="font-mono text-lg font-bold text-amber-300">{weapon.name}</h2><p className="text-sm text-slate-400">{weapon.role}</p></div>{hashButton(draft.sheet_hash, draft.sheet_short_hash)}</div>
            <div className="flex flex-wrap gap-1.5">{modes.map((entry, index) => <button type="button" key={entry.name} onClick={() => selectMode(index)} className={`rounded px-2 py-1.5 text-xs ${modeIndex === index ? "bg-amber-500 text-black" : "bg-slate-800 text-slate-300 hover:bg-slate-700"}`}>{entry.name}</button>)}</div>
            <div className="relative mt-4 flex h-[min(50vw,480px)] min-h-[240px] items-center justify-center overflow-hidden rounded border border-slate-700 bg-[radial-gradient(circle_at_center,#172c35,#05090c_75%)]">
              <div role="img" aria-label={`${weapon.name} ${mode.name} frame ${frame + 1}`} className="aspect-[4/3] h-full max-w-full bg-no-repeat" style={{ backgroundImage: `url(${asset(draft.sheet_file)})`, backgroundSize: "500% 500%", backgroundPosition: `${(cell % 5) * 25}% ${Math.floor(cell / 5) * 25}%` }} />
              <span className="pointer-events-none absolute left-1/2 top-1/2 h-3 w-3 -translate-x-1/2 -translate-y-1/2 border border-cyan-300/70" />
              <span className="absolute bottom-2 left-2 rounded bg-black/80 px-2 py-1 font-mono text-xs">{mode.name.toUpperCase()} · FRAME {frame + 1}/{mode.count}</span>
            </div>
            <div className="mt-3 flex flex-wrap items-center gap-2">
              <button type="button" onClick={() => setPlaying((value) => !value)} disabled={mode.count === 1} className="rounded bg-slate-800 p-2 disabled:opacity-40" aria-label={playing ? "Pause animation" : "Play animation"}>{playing ? <Pause size={16} /> : <Play size={16} />}</button>
              {Array.from({ length: mode.count }, (_, index) => <button type="button" key={index} onClick={() => { setFrame(index); setPlaying(false); }} className={`h-8 min-w-8 rounded font-mono text-xs ${frame === index ? "bg-amber-500 text-black" : "bg-slate-800"}`}>{index + 1}</button>)}
              <span className="ml-auto font-mono text-xs text-slate-400">{draft.cell_w}×{draft.cell_h} PER FRAME · 5×5 GRID</span>
            </div>
          </div>
          <div className="grid gap-3 sm:grid-cols-2">
            {[["Sprite sheet", draft.sheet_file, draft.sheet_hash, draft.sheet_short_hash], ["Aim reference", draft.aim_file, draft.aim_hash, draft.aim_short_hash]].map(([label, file, hash, short]) => file && <div key={label} className="min-w-0 rounded border border-slate-800 bg-[#101820] p-3"><p className="mb-2 font-mono text-xs uppercase text-slate-400">{label}</p><img src={asset(file)} alt={label} className="h-28 w-full rounded bg-black/50 object-contain" /><p className="mt-2 truncate font-mono text-[10px]" title={file}>{file}</p>{hash && short && <div className="mt-2">{hashButton(hash, short)}</div>}</div>)}
          </div>
        </section>
      </div> : <div><h2 className="mb-4 font-mono text-sm uppercase text-amber-300">{tab} · {filtered.length} assets</h2><div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-5 xl:grid-cols-6">
        {filtered.map((item) => {
          const image = /\.(png|jpe?g|webp|svg)$/.test(item.file);
          const audio = /\.(mp3|ogg|wav)$/.test(item.file);
          return <article key={item.file} className="min-w-0 rounded border border-slate-800 bg-[#101820] p-2">
            <div className="flex h-28 items-center justify-center overflow-hidden rounded bg-black/50">{image ? item.group === "textures" && item.seamless !== false ? <div className="h-full w-full" style={{ backgroundImage: `url(${asset(item.file)})`, backgroundRepeat: "repeat", backgroundSize: "50% 50%" }} /> : <img src={asset(item.file)} alt={item.name} loading="lazy" className="max-h-full max-w-full object-contain" /> : audio ? <audio controls preload="none" src={asset(item.file)} className="w-full" onPlay={(event) => { if (audioRef.current && audioRef.current !== event.currentTarget) audioRef.current.pause(); audioRef.current = event.currentTarget; }} /> : <Volume2 size={24} className="text-slate-500" />}</div>
            <p className="mt-2 truncate text-xs" title={item.name}>{item.name}</p><p className="truncate font-mono text-[10px] text-slate-500" title={item.file}>{item.file}</p>
            <div className="mt-2">{hashButton(item.hash, item.shortHash)}</div>
          </article>;
        })}
      </div></div>}
    </main>
  </div>;
}
