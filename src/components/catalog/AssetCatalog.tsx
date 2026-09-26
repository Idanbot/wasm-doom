import React, { useState, useEffect, useRef, useMemo } from "react";
import {
  Play,
  Pause,
  RotateCcw,
  Crosshair as CrosshairIcon,
  Copy,
  Check,
  Search,
  Zap,
  Flame,
  Shield,
  Skull,
  Package,
  Volume2,
  Layers,
  Sparkles,
  ArrowRight,
  Eye,
  Sliders,
  Hash,
  Terminal,
  ChevronDown,
  RefreshCw,
  Info,
  Grid,
  AlertTriangle,
  Radio,
} from "lucide-react";
import { asset } from "@/lib/asset";
import { Crosshair } from "../game/Crosshair";
import catalogDataRaw from "@/lib/asset-catalog-data.json";
import draftWeaponsRaw from "@/lib/draft-weapons-data.json";
import draftWeaponsV2Raw from "@/lib/draft-weapons-v2-data.json";
import draftAssetsV2Raw from "@/lib/draft-assets-v2-data.json";

export interface AssetItem {
  file: string;
  name: string;
  size: number;
  hash: string;
  shortHash: string;
  width?: number;
  height?: number;
}

export interface DraftWeaponData {
  id: string;
  name: string;
  slot: number;
  is_boss: boolean;
  alt_fire_name: string;
  alt_fire_desc: string;
  alt_available: boolean;
  ammo_visual: string;
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
  cols: number;
  rows: number;
  frames_total: number;
}

export interface WeaponRewriteSpec {
  slot: number;
  id: string;
  name: string;
  designation: string;
  role: string;
  lore: string;
  combatIdentity: string;
  magSize: number;
  ammoType: string;
  damage: string;
  sourceFiles: {
    idle: string;
    fire: string;
    reload: string;
    pickup: string;
  };
}

export const WEAPON_SPECS: WeaponRewriteSpec[] = [
  {
    slot: 8,
    id: "vr9",
    name: "VR-9 OVERRIDE",
    designation: "Gun 9 / Slot 8 (Boss Drop: Malik Veyran)",
    role: "Veyran rail · pierces the lane, then bursts",
    lore: "VR-9 OVERRIDE precision rail rifle captured from Malik Veyran, dark command-armor plating, amber charge chamber.",
    combatIdentity: "Boss superweapon. Pierces the entire lane for 120 damage, then detonates a 2.8m shockwave burst.",
    magSize: 4,
    ammoType: "Veyran Override Rod",
    damage: "120 + 2.8m AoE Burst",
    sourceFiles: {
      idle: "/game/weap_vr9.png",
      fire: "/game/weap_vr9_fire.png",
      reload: "/game/weap_vr9_reload.png",
      pickup: "/game/spr_gun_vr9.png",
    },
  },
  {
    slot: 9,
    id: "hc9",
    name: "HC-9 FORGE",
    designation: "Gun 10 / Slot 9 (Boss Drop: Hecate-9)",
    role: "HECATE cutter · wide beam and impact splash",
    lore: "HC-9 FORGE rotary electrical cannon captured from HECATE-9, central bore, cyan induction coils, ceramic heat shields.",
    combatIdentity: "Forge Warden cutter. Sweeps a wide cutting plasma arc with heavy impact splash across target groups.",
    magSize: 14,
    ammoType: "Plasma Induction Cells",
    damage: "40 (Wide Cone Sweep)",
    sourceFiles: {
      idle: "/game/weap_hc9.png",
      fire: "/game/weap_hc9_fire.png",
      reload: "/game/weap_hc9_reload.png",
      pickup: "/game/spr_gun_hc9.png",
    },
  },
  {
    slot: 10,
    id: "cm9",
    name: "CM-9 CHIMERA",
    designation: "Gun 11 / Slot 10 (Boss Drop: Chimera-9)",
    role: "Specimen fan · acid bursts and a short pool",
    lore: "CM-9 CHIMERA chemical projector captured from CHIMERA-9, off-white containment ceramic, two sealed green-fluid side cartridges.",
    combatIdentity: "Bio-containment weapon. Fires acidic clusters that burst on contact and leave a 4-second corrosive acid pool.",
    magSize: 5,
    ammoType: "Bio-Vitriol Cartridges",
    damage: "50 + 4s Corrosive Pool",
    sourceFiles: {
      idle: "/game/weap_cm9.png",
      fire: "/game/weap_cm9_fire.png",
      reload: "/game/weap_cm9_reload.png",
      pickup: "/game/spr_gun_cm9.png",
    },
  },
  {
    slot: 0,
    id: "mk23s",
    name: "MK23-S",
    designation: "Gun 1 / Slot 0 (Default Sidearm)",
    role: "Suppressed precision · 12 rounds",
    lore: "Compact suppressed military sidearm, squared slide, integral barrel shroud, amber status diode.",
    combatIdentity: "Starting precision weapon. Zero spread on initial shot, silent report, crisp sight recovery.",
    magSize: 12,
    ammoType: "9mm Ballistic",
    damage: "18 per shot",
    sourceFiles: {
      idle: "/game/weap_mk23s.png",
      fire: "/game/weap_mk23s_fire.png",
      reload: "/game/weap_mk23s_reload.png",
      pickup: "/game/spr_gun_mk23s.png",
    },
  },
  {
    slot: 1,
    id: "br12",
    name: "BR-12 BREAKER",
    designation: "Gun 2 / Slot 1 (Sector Breacher)",
    role: "8-shot breacher · heavy stagger",
    lore: "Short brutal pump shotgun, thick barrel shroud, visible pump grip, industrial breaching teeth.",
    combatIdentity: "Close-quarters heavy weapon. High pellet count, wide spread cone, guaranteed heavy stagger/flinch.",
    magSize: 8,
    ammoType: "12ga Mag-Slug / Buck",
    damage: "12 × 10 pellets",
    sourceFiles: {
      idle: "/game/weap_br12.png",
      fire: "/game/weap_br12_fire.png",
      reload: "/game/weap_br12_reload.png",
      pickup: "/game/spr_gun_br12.png",
    },
  },
  {
    slot: 2,
    id: "kx9",
    name: "KX-9 VECTOR",
    designation: "Gun 3 / Slot 2 (Sector Automatic)",
    role: "36-round PDW · controlled burst",
    lore: "Compact bullpup machine carbine, short barrel, box magazine, cyan digital ammunition counter.",
    combatIdentity: "High-cadence tracking weapon. Excellent against fast swarmers like hounds and hornets.",
    magSize: 36,
    ammoType: "Sub-caliber Kinetic",
    damage: "14 per round (900 RPM)",
    sourceFiles: {
      idle: "/game/weap_kx9.png",
      fire: "/game/weap_kx9_fire.png",
      reload: "/game/weap_kx9_reload.png",
      pickup: "/game/spr_gun_kx9.png",
    },
  },
  {
    slot: 3,
    id: "mr4",
    name: "MR-4 LONGBOW",
    designation: "Gun 4 / Slot 3 (Sector Penetrator)",
    role: "Magnetic penetrator · pierces 3",
    lore: "Heavy electromagnetic precision rifle, twin acceleration rails, narrow cyan charge channel, angular stock.",
    combatIdentity: "Sniper / penetrator. Pierces up to 3 hostiles in line of sight, creating an ionizing wake.",
    magSize: 5,
    ammoType: "Ferro-Dart + Capacitor",
    damage: "85 (Piercing ×3)",
    sourceFiles: {
      idle: "/game/weap_mr4.png",
      fire: "/game/weap_mr4_fire.png",
      reload: "/game/weap_mr4_reload.png",
      pickup: "/game/spr_gun_mr4.png",
    },
  },
  {
    slot: 4,
    id: "vlk6",
    name: "VLK-6 WARDEN",
    designation: "Gun 5 / Slot 4 (Sector Demolition)",
    role: "Guided micro-missile · blast radius",
    lore: "Shoulder-fired guided missile launcher, large rectangular 4-tube pod, side targeting optic, top carry rail.",
    combatIdentity: "Heavy explosive demolition. 4-cell salvo launcher with high splash damage for armored brutes.",
    magSize: 4,
    ammoType: "Micro-Ordnance Cell",
    damage: "140 (AoE 3.2m radius)",
    sourceFiles: {
      idle: "/game/weap_vlk6.png",
      fire: "/game/weap_vlk6_fire.png",
      reload: "/game/weap_vlk6_reload.png",
      pickup: "/game/spr_gun_vlk6.png",
    },
  },
  {
    slot: 5,
    id: "ax12",
    name: "AX-12 VOLT",
    designation: "Gun 6 / Slot 5 (Sector Energy)",
    role: "Electrical carbine · precision shock",
    lore: "Experimental arc caster, split copper induction prongs around glowing cyan capacitor, insulated body.",
    combatIdentity: "Chaining electrical assault. Shocks primary target and arcs to adjacent foes, briefly stunning.",
    magSize: 10,
    ammoType: "High-Coulomb Cells",
    damage: "35 (Arc Chain ×2)",
    sourceFiles: {
      idle: "/game/weap_ax12.png",
      fire: "/game/weap_ax12_fire.png",
      reload: "/game/weap_ax12_reload.png",
      pickup: "/game/spr_gun_ax12.png",
    },
  },
  {
    slot: 6,
    id: "m91",
    name: "M91 CYCLONE",
    designation: "Gun 7 / Slot 6 (Sector Heavy)",
    role: "Rotary cannon · sustained suppression",
    lore: "Heavy rotary cannon with six-barrel cluster, armored motor housing, belt-feed box, amber heat vents.",
    combatIdentity: "Extreme suppressive fire. Spools up to fire a devastating wall of lead, shredding multiple waves.",
    magSize: 90,
    ammoType: "Heavy Ballistic Link",
    damage: "22 (1200 RPM max spool)",
    sourceFiles: {
      idle: "/game/weap_m91.png",
      fire: "/game/weap_m91_fire.png",
      reload: "/game/weap_m91_reload.png",
      pickup: "/game/spr_gun_m91.png",
    },
  },
  {
    slot: 7,
    id: "hx8",
    name: "HX-8 PYRE",
    designation: "Gun 8 / Slot 7 (Sector Incendiary)",
    role: "Incendiary projector · leaves a burn",
    lore: "Compact incendiary projector with top fuel canister, amber pilot lamp and heat-shielded nozzle shroud.",
    combatIdentity: "Area denial flame weapon. Leaves a lingering 10-second ground fire pool that melts crowds.",
    magSize: 6,
    ammoType: "Thermite Fuel Canisters",
    damage: "45/s + 10s lingering hazard",
    sourceFiles: {
      idle: "/game/weap_hx8.png",
      fire: "/game/weap_hx8_fire.png",
      reload: "/game/weap_hx8_reload.png",
      pickup: "/game/spr_gun_hx8.png",
    },
  },
];

const catalogData: AssetItem[] = catalogDataRaw as AssetItem[];
const originalDraftWeapons: DraftWeaponData[] = draftWeaponsRaw as DraftWeaponData[];
const revisedDraftWeapons: DraftWeaponData[] = draftWeaponsV2Raw as DraftWeaponData[];
const revisedAssets: (AssetItem & { group: string })[] = draftAssetsV2Raw;
const V2_ROLES: Record<string, string> = {
  mr4: "Precision marksman rifle · pierces 3",
  ax12: "Practical electroshock carbine · precision shock",
  m91: "Belt-fed machine gun · sustained suppression",
};
const V2_AMMO: Record<string, string> = {
  mr4: "Precision Rifle Cartridge",
  m91: "Linked Ballistic Rounds",
};

// 5x5 Grid animation state mappings
export type DraftAnimMode =
  | "full_aim"
  | "half_aim"
  | "low_aim"
  | "empty_mag_aim"
  | "no_ammo_aim"
  | "fire_empty"
  | "pickup"
  | "reload"
  | "fire"
  | "alt_fire";

interface DraftAnimConfig {
  label: string;
  frames: number;
  startCell: number;
  fps: number;
  row: number;
  description: string;
}

const DRAFT_ANIMS: Record<DraftAnimMode, DraftAnimConfig> = {
  full_aim: {
    label: "Full Ammo Aim",
    frames: 1,
    startCell: 0,
    fps: 1,
    row: 0,
    description: "Basic frame: magazine/cell inserted and locked, static aim on center reticle with full ammo (100% display, all fuel rods glowing).",
  },
  half_aim: {
    label: "Half Ammo Aim",
    frames: 1,
    startCell: 1,
    fps: 1,
    row: 0,
    description: "Static aim with half ammo: visual indicators reflect 50% capacity (energy bar/text at 50%, 2 rods glowing; solid ballistic mags remain identical).",
  },
  low_aim: {
    label: "Low Ammo Aim",
    frames: 1,
    startCell: 2,
    fps: 1,
    row: 0,
    description: "Static aim with low ammo (15-20%): warning indicator active, caution diode illuminated, 1 rod flickering.",
  },
  empty_mag_aim: {
    label: "Empty Mag Aim",
    frames: 1,
    startCell: 3,
    fps: 1,
    row: 0,
    description: "Magazine depleted: empty receptacle / slide locked back, needs reload; dry-click on fire.",
  },
  no_ammo_aim: {
    label: "No Ammo Aim",
    frames: 1,
    startCell: 4,
    fps: 1,
    row: 0,
    description: "Total ammunition exhaustion: cannot reload and no magazine attached. Completely unpowered chassis, dark HUD.",
  },
  fire_empty: {
    label: "Fire Empty (Dry Fire)",
    frames: 1,
    startCell: 5,
    fps: 1,
    row: 1,
    description: "Dry-fire mechanical strike on empty chamber: striker/solenoid click, slight 1-2px jar backward, no muzzle flash.",
  },
  pickup: {
    label: "Pick-up / Equip",
    frames: 4,
    startCell: 6,
    fps: 6,
    row: 1,
    description: "Pick up from gun case on the floor (4f): hands reach into open foam cutout -> lift weapon upward -> inspect at chest height -> shoulder mount.",
  },
  reload: {
    label: "Reload Sequence",
    frames: 5,
    startCell: 10,
    fps: 6,
    row: 2,
    description: "Reload sequence (5f): drop/eject empty -> fetch fresh glowing cell -> slide into mag well -> slap lock contact spark -> power cycle to aim.",
  },
  fire: {
    label: "Primary Fire",
    frames: 5,
    startCell: 15,
    fps: 8,
    row: 3,
    description: "Primary fire (5f): trigger break muzzle blast -> peak recoil buck up-left -> casing/slug eject -> return stroke -> settle on reticle.",
  },
  alt_fire: {
    label: "Alt-Fire Special",
    frames: 5,
    startCell: 20,
    fps: 7,
    row: 4,
    description: "Special Right Mouse Button fire: charged singularity blast / burst / spread (identical to primary fire on standard ballistic weapons).",
  },
};

export function AssetCatalog({ onBackToGame }: { onBackToGame?: () => void }) {
  const [activeTab, setActiveTab] = useState<
    "weapons" | "items" | "enemies" | "particles" | "textures" | "audio" | "all"
  >("weapons");

  // Toggle between LIVE SET and DRAFT SET
  const [weaponSet, setWeaponSet] = useState<"draft" | "live">("draft");
  const [draftRevision, setDraftRevision] = useState<"v2" | "original">("v2");
  const draftWeapons = draftRevision === "v2" ? revisedDraftWeapons : originalDraftWeapons;
  const [selectedWeaponId, setSelectedWeaponId] = useState<string>("mk23s");
  const [searchQuery, setSearchQuery] = useState("");
  const [copiedHash, setCopiedHash] = useState<string | null>(null);

  // Draft 5x5 Animation Mode State
  const [draftMode, setDraftMode] = useState<DraftAnimMode>("full_aim");
  const [draftFrameIndex, setDraftFrameIndex] = useState(0);

  // Live Animation Mode State
  const [liveMode, setLiveMode] = useState<"aim" | "fire" | "reload" | "empty">("fire");
  const [liveFrameIndex, setLiveFrameIndex] = useState(0);

  // Shared Player Controls
  const [isPlaying, setIsPlaying] = useState(true);
  const [fps, setFps] = useState(6);
  const [showCrosshair, setShowCrosshair] = useState(true);
  const [showGridOverview, setShowGridOverview] = useState(false);

  const selectedSpec = useMemo(
    () => WEAPON_SPECS.find((w) => w.id === selectedWeaponId) || WEAPON_SPECS[0]!,
    [selectedWeaponId]
  );

  const selectedDraft = useMemo(
    () => draftWeapons.find((w) => w.id === selectedWeaponId) || draftWeapons[0]!,
    [selectedWeaponId]
  );

  // Current animation config for draft
  const currentDraftConfig = DRAFT_ANIMS[draftMode];
  const activeCellIndex = currentDraftConfig.startCell + draftFrameIndex;

  // Live animation total frames
  const liveTotalFrames = useMemo(() => {
    switch (liveMode) {
      case "aim":
        return 1;
      case "fire":
        return 3;
      case "reload":
        return 4;
      case "empty":
        return 1;
    }
  }, [liveMode]);

  // Draft Animation Ticker
  useEffect(() => {
    if (!isPlaying || weaponSet !== "draft" || currentDraftConfig.frames <= 1) return;
    const interval = setInterval(() => {
      setDraftFrameIndex((prev) => (prev + 1) % currentDraftConfig.frames);
    }, 1000 / fps);
    return () => clearInterval(interval);
  }, [isPlaying, weaponSet, currentDraftConfig, fps]);

  // Live Animation Ticker
  useEffect(() => {
    if (!isPlaying || weaponSet !== "live" || liveTotalFrames <= 1) return;
    const interval = setInterval(() => {
      setLiveFrameIndex((prev) => (prev + 1) % liveTotalFrames);
    }, 1000 / fps);
    return () => clearInterval(interval);
  }, [isPlaying, weaponSet, liveTotalFrames, fps]);

  // Reset frames on weapon/mode change
  useEffect(() => {
    setDraftFrameIndex(0);
    setLiveFrameIndex(0);
  }, [draftMode, liveMode, selectedWeaponId, weaponSet]);

  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard.writeText(text);
    setCopiedHash(label);
    setTimeout(() => setCopiedHash(null), 2000);
  };

  const getAsset = (path: string): AssetItem | undefined => {
    return catalogData.find((item) => item.file === path);
  };

  const filteredAssets = useMemo(() => {
    let items: AssetItem[] = [...catalogData, ...revisedAssets];
    if (activeTab === "items") {
      items = items.filter(
        (i) =>
          i.name.startsWith("spr_gun_") ||
          i.name.startsWith("spr_med") ||
          i.name.startsWith("spr_armor") ||
          i.name.startsWith("spr_ammo") ||
          i.name.startsWith("spr_ord") ||
          i.name.startsWith("spr_crate") ||
          i.name.startsWith("spr_barrel") ||
          i.name.startsWith("spr_lamp") ||
          i.name.startsWith("spr_chain") ||
          ("group" in i && i.group === "items")
      );
    } else if (activeTab === "enemies") {
      items = items.filter((i) => i.name.startsWith("enemy_"));
    } else if (activeTab === "particles") {
      items = items.filter(
        (i) =>
          i.name.startsWith("spr_flame") ||
          i.name.startsWith("spr_muzzle") ||
          i.name.startsWith("spr_impact") ||
          i.name.startsWith("spr_ball") ||
          ("group" in i && i.group === "particles")
      );
    } else if (activeTab === "textures") {
      items = items.filter(
        (i) => i.name.startsWith("wall_") || i.name.startsWith("floor_") || i.name.startsWith("ceil_") || ("group" in i && i.group === "textures")
      );
    } else if (activeTab === "audio") {
      items = items.filter((i) => i.file.includes("/sfx/") || i.file.includes("/voices/") || i.file.includes("/music/"));
    }

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      items = items.filter(
        (i) => i.name.toLowerCase().includes(q) || i.hash.toLowerCase().includes(q) || i.file.toLowerCase().includes(q)
      );
    }

    return items;
  }, [activeTab, searchQuery]);

  return (
    <div className="min-h-screen bg-[#070b0e] text-slate-200 font-sans flex flex-col selection:bg-amber-500/30 selection:text-amber-200">
      {/* Header bar */}
      <header className="sticky top-0 z-50 bg-[#090e13]/95 backdrop-blur-md border-b border-amber-950/40 px-4 py-3 flex flex-wrap items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded bg-gradient-to-br from-amber-500 to-amber-700 flex items-center justify-center text-black font-black text-lg shadow-lg shadow-amber-500/20">
            B
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-sm font-bold tracking-widest text-amber-400 uppercase font-mono">
                BLACKSITE // ASSET ARCHIVE
              </h1>
              <span className="px-1.5 py-0.5 text-[10px] uppercase font-mono tracking-wider bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 rounded">
                DEV ONLY
              </span>
            </div>
            <p className="text-[11px] text-slate-400 font-mono">
              FPS Viewmodel Architecture · 5×5 25-Frame High Quality Drafts · Reticle Center-Aligned
            </p>
          </div>
        </div>

        {/* Global actions */}
        <div className="flex items-center gap-3">
          <div className="relative">
            <Search className="w-4 h-4 text-slate-400 absolute left-2.5 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              placeholder="Search assets or hash..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="bg-black/50 border border-slate-700/60 rounded px-2.5 py-1.5 pl-8 text-xs text-slate-200 placeholder:text-slate-500 focus:outline-none focus:border-amber-500/60 w-48 sm:w-64 font-mono"
            />
          </div>

          {onBackToGame && (
            <button
              onClick={onBackToGame}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded bg-amber-500 hover:bg-amber-400 text-black font-bold text-xs uppercase tracking-wider transition-all shadow-md shadow-amber-500/20 cursor-pointer"
            >
              <RotateCcw className="w-3.5 h-3.5" />
              <span>Back to Game</span>
            </button>
          )}
        </div>
      </header>

      {/* Category Navigation Tabs */}
      <div className="bg-[#0b1016] border-b border-slate-800/80 px-4 py-2 flex items-center justify-between gap-4 overflow-x-auto text-xs font-mono">
        <div className="flex items-center gap-2">
          <button
            onClick={() => setActiveTab("weapons")}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded transition-all cursor-pointer whitespace-nowrap ${
              activeTab === "weapons"
                ? "bg-amber-500/20 text-amber-300 border border-amber-500/40"
                : "text-slate-400 hover:text-slate-200 hover:bg-slate-800/40"
            }`}
          >
            <CrosshairIcon className="w-3.5 h-3.5" />
            <span>WEAPONS (GUN 1–11)</span>
            <span className="px-1 text-[10px] rounded bg-black/40 text-amber-400">11</span>
          </button>

          <button
            onClick={() => setActiveTab("items")}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded transition-all cursor-pointer whitespace-nowrap ${
              activeTab === "items"
                ? "bg-amber-500/20 text-amber-300 border border-amber-500/40"
                : "text-slate-400 hover:text-slate-200 hover:bg-slate-800/40"
            }`}
          >
            <Package className="w-3.5 h-3.5" />
            <span>PICKUPS & ITEMS</span>
            <span className="px-1 text-[10px] rounded bg-black/40 text-slate-300">41</span>
          </button>

          <button
            onClick={() => setActiveTab("enemies")}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded transition-all cursor-pointer whitespace-nowrap ${
              activeTab === "enemies"
                ? "bg-amber-500/20 text-amber-300 border border-amber-500/40"
                : "text-slate-400 hover:text-slate-200 hover:bg-slate-800/40"
            }`}
          >
            <Skull className="w-3.5 h-3.5" />
            <span>ENEMIES & BOSSES</span>
            <span className="px-1 text-[10px] rounded bg-black/40 text-slate-300">105</span>
          </button>

          <button
            onClick={() => setActiveTab("particles")}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded transition-all cursor-pointer whitespace-nowrap ${
              activeTab === "particles"
                ? "bg-amber-500/20 text-amber-300 border border-amber-500/40"
                : "text-slate-400 hover:text-slate-200 hover:bg-slate-800/40"
            }`}
          >
            <Sparkles className="w-3.5 h-3.5" />
            <span>PARTICLES & FX</span>
            <span className="px-1 text-[10px] rounded bg-black/40 text-slate-300">24</span>
          </button>

          <button
            onClick={() => setActiveTab("textures")}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded transition-all cursor-pointer whitespace-nowrap ${
              activeTab === "textures"
                ? "bg-amber-500/20 text-amber-300 border border-amber-500/40"
                : "text-slate-400 hover:text-slate-200 hover:bg-slate-800/40"
            }`}
          >
            <Layers className="w-3.5 h-3.5" />
            <span>TEXTURES</span>
            <span className="px-1 text-[10px] rounded bg-black/40 text-slate-300">47</span>
          </button>

          <button
            onClick={() => setActiveTab("audio")}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded transition-all cursor-pointer whitespace-nowrap ${
              activeTab === "audio"
                ? "bg-amber-500/20 text-amber-300 border border-amber-500/40"
                : "text-slate-400 hover:text-slate-200 hover:bg-slate-800/40"
            }`}
          >
            <Volume2 className="w-3.5 h-3.5" />
            <span>AUDIO & VOICES</span>
            <span className="px-1 text-[10px] rounded bg-black/40 text-slate-300">100</span>
          </button>
        </div>

        {/* Live vs Draft Set Toggle Switch */}
        {activeTab === "weapons" && (
          <div className="flex items-center gap-2 shrink-0">
            {weaponSet === "draft" && (
              <div className="flex items-center bg-black/60 p-1 rounded border border-slate-800">
                <button onClick={() => setDraftRevision("v2")} className={`px-2 py-1 rounded font-bold ${draftRevision === "v2" ? "bg-amber-500 text-black" : "text-slate-400"}`}>NEW V2</button>
                <button onClick={() => setDraftRevision("original")} className={`px-2 py-1 rounded font-bold ${draftRevision === "original" ? "bg-amber-500 text-black" : "text-slate-400"}`}>EARLIER</button>
              </div>
            )}
          <div className="flex items-center bg-black/60 p-1 rounded border border-slate-800">
            <span className="text-[10px] text-slate-400 mr-2 px-1">SET:</span>
            <button
              onClick={() => setWeaponSet("draft")}
              className={`px-3 py-1 rounded text-xs font-bold transition-all cursor-pointer flex items-center gap-1.5 ${
                weaponSet === "draft"
                  ? "bg-amber-500 text-black shadow-md shadow-amber-500/20"
                  : "text-slate-400 hover:text-white"
              }`}
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>DRAFT SET (5×5 25-FRAME)</span>
            </button>
            <button
              onClick={() => setWeaponSet("live")}
              className={`px-3 py-1 rounded text-xs font-bold transition-all cursor-pointer flex items-center gap-1.5 ${
                weaponSet === "live"
                  ? "bg-cyan-500 text-black shadow-md shadow-cyan-500/20"
                  : "text-slate-400 hover:text-white"
              }`}
            >
              <Radio className="w-3.5 h-3.5" />
              <span>LIVE SET (ENGINE)</span>
            </button>
          </div>
          </div>
        )}
      </div>

      {/* Main Content */}
      <main className="flex-1 p-4 lg:p-6 overflow-y-auto max-w-[1600px] w-full mx-auto">
        {activeTab === "weapons" ? (
          <div className="grid grid-cols-1 xl:grid-cols-12 gap-6">
            {/* Left sidebar: Gun 1 to 11 selector */}
            <div className="xl:col-span-4 flex flex-col gap-3">
              <div className="bg-[#0b1016] border border-slate-800 rounded-lg p-3">
                <div className="flex items-center justify-between mb-2">
                  <h2 className="text-xs font-bold text-amber-400 font-mono uppercase tracking-wider flex items-center gap-1.5">
                    <CrosshairIcon className="w-4 h-4" />
                    SELECT WEAPON (GUN 1–11)
                  </h2>
                  <span className="text-[10px] text-slate-400 font-mono">
                    {weaponSet === "draft" ? "DRAFT FPS" : "LIVE ENGINE"}
                  </span>
                </div>

                <div className="flex flex-col gap-1.5 max-h-[240px] sm:max-h-[640px] overflow-y-auto pr-1">
                  {WEAPON_SPECS.map((w) => {
                    const isSelected = w.id === selectedWeaponId;
                    const draftItem = draftWeapons.find((d) => d.id === w.id);
                    const idleAsset = getAsset(w.sourceFiles.idle);
                    const thumbImg =
                      weaponSet === "draft" && draftItem ? asset(draftItem.aim_file) : asset(w.sourceFiles.idle);

                    return (
                      <button
                        key={w.id}
                        onClick={() => setSelectedWeaponId(w.id)}
                        className={`text-left p-2.5 rounded transition-all cursor-pointer border flex items-center justify-between ${
                          isSelected
                            ? "bg-amber-500/15 border-amber-500/50 text-white shadow-md shadow-amber-500/10"
                            : "bg-black/30 border-slate-800/80 hover:bg-slate-800/40 hover:border-slate-700 text-slate-300"
                        }`}
                      >
                        <div className="flex items-center gap-3">
                          <div className="w-12 h-9 rounded bg-black/60 border border-slate-800 flex items-center justify-center p-1 shrink-0 overflow-hidden relative">
                            <img
                              src={thumbImg}
                              alt={w.name}
                              className="w-full h-full object-contain"
                            />
                            {draftItem?.is_boss && (
                              <span className="absolute top-0.5 right-0.5 w-1.5 h-1.5 rounded-full bg-amber-400 animate-ping" />
                            )}
                          </div>
                          <div>
                            <div className="flex items-center gap-1.5">
                              <span className="font-bold text-xs tracking-wide text-white">{w.name}</span>
                              {draftItem?.is_boss ? (
                                <span className="text-[9px] font-mono px-1 py-0.2 rounded bg-amber-950/80 border border-amber-500/40 text-amber-300">
                                  BOSS
                                </span>
                              ) : (
                                <span className="text-[9px] font-mono px-1 py-0.2 rounded bg-slate-800 text-slate-400">
                                  Slot {w.slot}
                                </span>
                              )}
                            </div>
                            <div className="text-[11px] text-slate-400 line-clamp-1">{weaponSet === "draft" && draftRevision === "v2" ? V2_ROLES[w.id] || w.role : w.role}</div>
                          </div>
                        </div>

                        <div className="text-right font-mono text-[10px] text-slate-400 shrink-0">
                          {weaponSet === "draft" && draftItem ? (
                            <span className="px-1 py-0.5 rounded bg-slate-900 border border-slate-800 text-amber-400">
                              #{draftItem.sheet_short_hash}
                            </span>
                          ) : idleAsset ? (
                            <span className="px-1 py-0.5 rounded bg-slate-900 border border-slate-800 text-cyan-400">
                              #{idleAsset.shortHash}
                            </span>
                          ) : null}
                        </div>
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Rework feedback prompt generator */}
              <div className="bg-[#0b1016] border border-amber-950/50 rounded-lg p-3.5">
                <div className="flex items-center gap-2 mb-2 text-amber-400 font-mono text-xs font-bold uppercase">
                  <Terminal className="w-4 h-4" />
                  REWORK FEEDBACK PROMPT
                </div>
                <p className="text-[11px] text-slate-400 mb-2">
                  Use this prompt format with hash to request specific frame changes in chat:
                </p>

                {(() => {
                  const hash =
                    weaponSet === "draft"
                      ? selectedDraft.sheet_short_hash
                      : getAsset(selectedSpec.sourceFiles.idle)?.shortHash ?? "hash";
                  const promptSnippet = `Rework #${hash} (${selectedSpec.name} ${weaponSet.toUpperCase()}): please adjust recoil angle on frame 2 and make the alt-fire muzzle bloom brighter.`;

                  return (
                    <div className="space-y-2">
                      <div className="bg-black/60 border border-slate-800 rounded p-2 text-[11px] font-mono text-amber-300/90 break-all select-all">
                        {promptSnippet}
                      </div>

                      <button
                        onClick={() => copyToClipboard(promptSnippet, "rework-prompt")}
                        className="w-full flex items-center justify-center gap-2 py-1.5 px-3 rounded bg-amber-500/20 hover:bg-amber-500/30 border border-amber-500/40 text-amber-300 font-mono text-xs tracking-wider transition-all cursor-pointer"
                      >
                        {copiedHash === "rework-prompt" ? (
                          <>
                            <Check className="w-3.5 h-3.5 text-green-400" />
                            <span>COPIED PROMPT WITH HASH!</span>
                          </>
                        ) : (
                          <>
                            <Copy className="w-3.5 h-3.5" />
                            <span>COPY REWORK PROMPT WITH HASH</span>
                          </>
                        )}
                      </button>
                    </div>
                  );
                })()}
              </div>
            </div>

            {/* Right main area: Interactive animation player + detailed rewrite spec */}
            <div className="xl:col-span-8 flex flex-col gap-6">
              {/* Weapon Header */}
              <div className="bg-[#0b1016] border border-slate-800 rounded-lg p-4">
                <div className="flex flex-wrap items-start justify-between gap-4 border-b border-slate-800/80 pb-3 mb-3">
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="px-2 py-0.5 text-xs font-mono font-bold bg-amber-500/20 text-amber-400 border border-amber-500/40 rounded">
                        {selectedSpec.designation}
                      </span>
                      <h2 className="text-xl font-black tracking-wider text-white font-mono">
                        {selectedSpec.name}
                      </h2>
                      {weaponSet === "draft" && (
                        <span className="px-2 py-0.5 text-[10px] font-mono bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 rounded font-bold">
                          DRAFT 5×5 (25F)
                        </span>
                      )}
                    </div>
                    <p className="text-xs text-slate-300 mt-1">{weaponSet === "draft" && draftRevision === "v2" ? V2_ROLES[selectedSpec.id] || selectedSpec.role : selectedSpec.role}</p>
                    {weaponSet === "draft" && draftRevision === "v2" && (
                      <p className="text-[11px] text-amber-200/80 mt-1">Individual reload and pickup poses; inspect all 25 frames.</p>
                    )}
                  </div>

                  <div className="flex items-center gap-3 font-mono text-xs">
                    <div className="bg-black/40 border border-slate-800 rounded px-2.5 py-1">
                      <span className="text-slate-400">MAG: </span>
                      <span className="font-bold text-amber-400">{selectedSpec.magSize}</span>
                    </div>
                    <div className="bg-black/40 border border-slate-800 rounded px-2.5 py-1">
                      <span className="text-slate-400">AMMO: </span>
                      <span className="font-bold text-cyan-400">{weaponSet === "draft" && draftRevision === "v2" ? V2_AMMO[selectedSpec.id] || selectedSpec.ammoType : selectedSpec.ammoType}</span>
                    </div>
                    <div className="bg-black/40 border border-slate-800 rounded px-2.5 py-1">
                      <span className="text-slate-400">DMG: </span>
                      <span className="font-bold text-slate-200">{selectedSpec.damage}</span>
                    </div>
                  </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
                  <div className="bg-black/40 border border-slate-800/80 rounded p-2.5">
                    <span className="text-amber-400 font-mono font-bold flex items-center gap-1.5 mb-1">
                      <Zap className="w-3.5 h-3.5" />
                      ALT-FIRE SPECIAL (RIGHT MOUSE BUTTON):
                    </span>
                    <div className="text-slate-200 font-bold">{selectedDraft.alt_fire_name}</div>
                    <div className="text-slate-400 text-[11px] mt-0.5">{selectedDraft.alt_fire_desc}</div>
                    <div className="mt-1.5">
                      {selectedDraft.alt_available ? (
                        <span className="px-1.5 py-0.5 rounded text-[10px] font-mono bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
                          AVAILABLE
                        </span>
                      ) : (
                        <span className="px-1.5 py-0.5 rounded text-[10px] font-mono bg-slate-800 text-slate-400 border border-slate-700">
                          UNAVAILABLE (BULLET GUN)
                        </span>
                      )}
                    </div>
                  </div>

                  <div className="bg-black/40 border border-slate-800/80 rounded p-2.5">
                    <span className="text-cyan-400 font-mono font-bold flex items-center gap-1.5 mb-1">
                      <Info className="w-3.5 h-3.5" />
                      AMMO VISUAL SPRITE BEHAVIOR:
                    </span>
                    <div className="text-slate-300 text-[11px] leading-relaxed">
                      {selectedDraft.ammo_visual}
                    </div>
                  </div>
                </div>
              </div>

              {/* Viewport Canvas Screen */}
              <div className="bg-[#0b1016] border border-slate-800 rounded-lg overflow-hidden flex flex-col">
                {/* Mode Selector Toolbar */}
                <div className="bg-black/50 border-b border-slate-800/80 p-3 flex flex-wrap items-center justify-between gap-3">
                  {weaponSet === "draft" ? (
                    /* Draft 5x5 Mode Buttons */
                    <div className="flex flex-wrap items-center gap-1 font-mono text-[11px]">
                      <span className="text-slate-400 mr-1">STATE:</span>
                      <button
                        onClick={() => setDraftMode("full_aim")}
                        className={`px-2 py-1 rounded transition-all cursor-pointer ${
                          draftMode === "full_aim"
                            ? "bg-amber-500 text-black font-bold"
                            : "bg-slate-800/70 text-slate-300 hover:bg-slate-700"
                        }`}
                      >
                        Full Aim (1f)
                      </button>
                      <button
                        onClick={() => setDraftMode("half_aim")}
                        className={`px-2 py-1 rounded transition-all cursor-pointer ${
                          draftMode === "half_aim"
                            ? "bg-amber-500 text-black font-bold"
                            : "bg-slate-800/70 text-slate-300 hover:bg-slate-700"
                        }`}
                      >
                        Half Aim (1f)
                      </button>
                      <button
                        onClick={() => setDraftMode("low_aim")}
                        className={`px-2 py-1 rounded transition-all cursor-pointer ${
                          draftMode === "low_aim"
                            ? "bg-amber-500 text-black font-bold"
                            : "bg-slate-800/70 text-slate-300 hover:bg-slate-700"
                        }`}
                      >
                        Low Aim (1f)
                      </button>
                      <button
                        onClick={() => setDraftMode("empty_mag_aim")}
                        className={`px-2 py-1 rounded transition-all cursor-pointer ${
                          draftMode === "empty_mag_aim"
                            ? "bg-amber-500 text-black font-bold"
                            : "bg-slate-800/70 text-slate-300 hover:bg-slate-700"
                        }`}
                      >
                        Empty Mag (1f)
                      </button>
                      <button
                        onClick={() => setDraftMode("no_ammo_aim")}
                        className={`px-2 py-1 rounded transition-all cursor-pointer ${
                          draftMode === "no_ammo_aim"
                            ? "bg-amber-500 text-black font-bold"
                            : "bg-slate-800/70 text-slate-300 hover:bg-slate-700"
                        }`}
                      >
                        No Ammo (1f)
                      </button>
                      <button
                        onClick={() => setDraftMode("fire_empty")}
                        className={`px-2 py-1 rounded transition-all cursor-pointer ${
                          draftMode === "fire_empty"
                            ? "bg-amber-500 text-black font-bold"
                            : "bg-slate-800/70 text-slate-300 hover:bg-slate-700"
                        }`}
                      >
                        Dry Fire (1f)
                      </button>
                      <button
                        onClick={() => setDraftMode("pickup")}
                        className={`px-2 py-1 rounded transition-all cursor-pointer ${
                          draftMode === "pickup"
                            ? "bg-amber-500 text-black font-bold"
                            : "bg-slate-800/70 text-slate-300 hover:bg-slate-700"
                        }`}
                      >
                        Pick-up (4f)
                      </button>
                      <button
                        onClick={() => setDraftMode("reload")}
                        className={`px-2 py-1 rounded transition-all cursor-pointer ${
                          draftMode === "reload"
                            ? "bg-amber-500 text-black font-bold"
                            : "bg-slate-800/70 text-slate-300 hover:bg-slate-700"
                        }`}
                      >
                        Reload (5f)
                      </button>
                      <button
                        onClick={() => setDraftMode("fire")}
                        className={`px-2 py-1 rounded transition-all cursor-pointer ${
                          draftMode === "fire"
                            ? "bg-amber-500 text-black font-bold"
                            : "bg-slate-800/70 text-slate-300 hover:bg-slate-700"
                        }`}
                      >
                        Fire (5f)
                      </button>
                      <button
                        onClick={() => setDraftMode("alt_fire")}
                        className={`px-2 py-1 rounded transition-all cursor-pointer ${
                          draftMode === "alt_fire"
                            ? "bg-amber-500 text-black font-bold"
                            : "bg-slate-800/70 text-slate-300 hover:bg-slate-700"
                        }`}
                      >
                        Alt-Fire (5f)
                      </button>
                    </div>
                  ) : (
                    /* Live Set Mode Buttons */
                    <div className="flex items-center gap-1.5 font-mono text-xs">
                      <span className="text-slate-400 mr-1 text-[11px]">STATE:</span>
                      <button
                        onClick={() => setLiveMode("aim")}
                        className={`px-2.5 py-1 rounded transition-all cursor-pointer ${
                          liveMode === "aim"
                            ? "bg-cyan-500 text-black font-bold"
                            : "bg-slate-800/70 text-slate-300 hover:bg-slate-700"
                        }`}
                      >
                        AIM (1f)
                      </button>
                      <button
                        onClick={() => setLiveMode("fire")}
                        className={`px-2.5 py-1 rounded transition-all cursor-pointer ${
                          liveMode === "fire"
                            ? "bg-cyan-500 text-black font-bold"
                            : "bg-slate-800/70 text-slate-300 hover:bg-slate-700"
                        }`}
                      >
                        FIRING (3f)
                      </button>
                      <button
                        onClick={() => setLiveMode("reload")}
                        className={`px-2.5 py-1 rounded transition-all cursor-pointer ${
                          liveMode === "reload"
                            ? "bg-cyan-500 text-black font-bold"
                            : "bg-slate-800/70 text-slate-300 hover:bg-slate-700"
                        }`}
                      >
                        RELOAD (4f)
                      </button>
                      <button
                        onClick={() => setLiveMode("empty")}
                        className={`px-2.5 py-1 rounded transition-all cursor-pointer ${
                          liveMode === "empty"
                            ? "bg-cyan-500 text-black font-bold"
                            : "bg-slate-800/70 text-slate-300 hover:bg-slate-700"
                        }`}
                      >
                        EMPTY (1f)
                      </button>
                    </div>
                  )}

                  {/* Player Controls */}
                  <div className="flex items-center gap-3">
                    <button
                      onClick={() => setShowCrosshair(!showCrosshair)}
                      className={`flex items-center gap-1 text-xs px-2 py-1 rounded font-mono transition-all cursor-pointer border ${
                        showCrosshair
                          ? "bg-cyan-500/20 text-cyan-300 border-cyan-500/40"
                          : "bg-black/30 text-slate-400 border-slate-800 hover:text-slate-200"
                      }`}
                    >
                      <CrosshairIcon className="w-3.5 h-3.5" />
                      <span>Reticle</span>
                    </button>

                    {weaponSet === "draft" && (
                      <button
                        onClick={() => setShowGridOverview(!showGridOverview)}
                        className={`flex items-center gap-1 text-xs px-2 py-1 rounded font-mono transition-all cursor-pointer border ${
                          showGridOverview
                            ? "bg-amber-500/20 text-amber-300 border-amber-500/40"
                            : "bg-black/30 text-slate-400 border-slate-800 hover:text-slate-200"
                        }`}
                      >
                        <Grid className="w-3.5 h-3.5" />
                        <span>5×5 Grid Sheet</span>
                      </button>
                    )}

                    <div className="flex items-center gap-1.5 font-mono text-xs">
                      <span className="text-slate-400 text-[11px]">FPS:</span>
                      <input
                        type="range"
                        min="2"
                        max="14"
                        value={fps}
                        onChange={(e) => setFps(Number(e.target.value))}
                        style={{ caretColor: "transparent" }}
                        className="w-16 accent-amber-500 cursor-pointer"
                      />
                      <span className="text-amber-400 w-4">{fps}</span>
                    </div>

                    <button
                      onClick={() => setIsPlaying(!isPlaying)}
                      className="p-1.5 rounded bg-slate-800 hover:bg-slate-700 text-white cursor-pointer"
                      title={isPlaying ? "Pause" : "Play"}
                    >
                      {isPlaying ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4" />}
                    </button>
                  </div>
                </div>

                {/* Viewport Canvas Screen: Reticle is in the Center, Gun is held from below and right */}
                <div className="relative w-full h-[440px] bg-gradient-to-b from-[#0b1219] to-[#05080c] flex items-center justify-center overflow-hidden border-b border-slate-800 select-none">
                  {/* Subtle target grid backdrop */}
                  <div
                    className="absolute inset-0 opacity-15 pointer-events-none"
                    style={{
                      backgroundImage:
                        "radial-gradient(#38bdf8 1px, transparent 1px), linear-gradient(to right, #1e293b 1px, transparent 1px), linear-gradient(to bottom, #1e293b 1px, transparent 1px)",
                      backgroundSize: "24px 24px, 48px 48px, 48px 48px",
                    }}
                  />

                  {/* 1. FPS Center Aim Reticle: Exact in-game Crosshair pinned at center (50% X, 50% Y) */}
                  {showCrosshair && (
                    <div className="absolute inset-0 pointer-events-none z-30 text-white flex items-center justify-center">
                      {/* Subtle alignment crosshair lines across the viewport to show exact center */}
                      <div className="absolute left-1/2 top-0 bottom-0 w-px border-l border-dashed border-cyan-500/25 pointer-events-none" />
                      <div className="absolute top-1/2 left-0 right-0 h-px border-t border-dashed border-cyan-500/25 pointer-events-none" />

                      {/* Exact in-game Crosshair */}
                      <Crosshair flash={0} spread={0} danger={false} />

                      {/* Aim sightline guide badge */}
                      <div className="absolute left-1/2 top-1/2 -translate-x-1/2 translate-y-6 whitespace-nowrap font-mono text-[9px] text-cyan-300 uppercase tracking-widest bg-black/85 px-2 py-0.5 rounded border border-cyan-500/40 shadow-lg pointer-events-none">
                        IN-GAME SIGHTLINE (50% × 50%)
                      </div>
                    </div>
                  )}

                  {/* 2. Gun Viewmodel: Held from below and right side of HUD */}
                  <div className={`absolute inset-0 pointer-events-none flex overflow-hidden z-10 ${weaponSet === "draft" ? "items-center justify-center" : "items-end justify-end"}`}>
                    {weaponSet === "draft" ? (
                      /* Draft 5x5 Sheet Cell Rendering */
                      <div
                        className="w-[512px] h-[384px] shrink-0 scale-[.68] sm:scale-100 transition-transform duration-75"
                        style={{
                          backgroundImage: `url(${asset(selectedDraft.sheet_file)})`,
                          backgroundSize: "500% 500%",
                          // Calculate background position: (col * 25%, row * 25%)
                          backgroundPosition: `${(activeCellIndex % 5) * 25}% ${Math.floor(activeCellIndex / 5) * 25}%`,
                          backgroundRepeat: "no-repeat",
                          // Position gun anchored from bottom-right, barrel pointing up toward the center reticle
                          transform: "none",
                        }}
                      />
                    ) : (
                      /* Live Engine Set Rendering */
                      (() => {
                        let bgUrl = asset(selectedSpec.sourceFiles.idle);
                        let bgSize = "contain";
                        let bgPos = "center bottom";

                        if (liveMode === "fire") {
                          bgUrl = asset(selectedSpec.sourceFiles.fire);
                          bgSize = "200% 200%";
                          const positions = ["0% 0%", "100% 0%", "0% 100%"];
                          bgPos = positions[liveFrameIndex] || "0% 0%";
                        } else if (liveMode === "reload") {
                          bgUrl = asset(selectedSpec.sourceFiles.reload);
                          if (selectedSpec.id === "mk23s") {
                            bgSize = "200% 200%";
                            const positions = ["0% 0%", "100% 0%", "0% 100%", "100% 100%"];
                            bgPos = positions[liveFrameIndex] || "0% 0%";
                          } else {
                            bgSize = "400% 200%";
                            const col = liveFrameIndex % 4;
                            const row = Math.floor(liveFrameIndex / 4);
                            bgPos = `${col * 33.333}% ${row * 100}%`;
                          }
                        }

                        return (
                          <div
                            className="w-[600px] h-[360px] origin-bottom-center transition-transform duration-75"
                            style={{
                              backgroundImage: `url(${bgUrl})`,
                              backgroundSize: bgSize,
                              backgroundPosition: bgPos,
                              backgroundRepeat: "no-repeat",
                              transform: "translate(-15%, 8%)",
                            }}
                          />
                        );
                      })()
                    )}
                  </div>

                  {/* Frame indicator badge */}
                  <div className="absolute bottom-3 left-3 z-40 bg-black/80 border border-slate-800 rounded px-2.5 py-1 font-mono text-xs flex items-center gap-2">
                    <span className="text-slate-400 uppercase text-[10px]">
                      {weaponSet === "draft" ? currentDraftConfig.label : liveMode}
                    </span>
                    <span className="text-amber-400 font-bold">
                      FRAME{" "}
                      {weaponSet === "draft"
                        ? `${draftFrameIndex + 1} / ${currentDraftConfig.frames}`
                        : `${liveFrameIndex + 1} / ${liveTotalFrames}`}
                    </span>
                    {weaponSet === "draft" && (
                      <span className="text-[10px] text-slate-500">
                        (Cell {activeCellIndex}: R{Math.floor(activeCellIndex / 5)}, C{activeCellIndex % 5})
                      </span>
                    )}
                  </div>

                  {/* Status alert badges */}
                  {weaponSet === "draft" && draftMode === "empty_mag_aim" && (
                    <div className="absolute top-3 right-3 z-40 bg-red-950/80 border border-red-500/50 rounded px-3 py-1 font-mono text-xs text-red-300 font-bold animate-pulse flex items-center gap-1.5">
                      <div className="w-2 h-2 rounded-full bg-red-500" />
                      SLIDE LOCKED // EMPTY MAG (1 FRAME)
                    </div>
                  )}

                  {weaponSet === "draft" && draftMode === "no_ammo_aim" && (
                    <div className="absolute top-3 right-3 z-40 bg-slate-900/90 border border-slate-700 rounded px-3 py-1 font-mono text-xs text-slate-300 font-bold flex items-center gap-1.5">
                      <AlertTriangle className="w-3.5 h-3.5 text-amber-500" />
                      ZERO RESERVE // NO AMMO (1 FRAME)
                    </div>
                  )}
                </div>

                {/* Scrubber & Details Bar */}
                <div className="p-3 bg-black/40 flex flex-col gap-2">
                  <div className="flex items-center gap-3">
                    <span className="text-[11px] font-mono text-slate-400">SCRUB:</span>
                    <div className="flex-1 flex gap-1">
                      {Array.from({
                        length: weaponSet === "draft" ? currentDraftConfig.frames : liveTotalFrames,
                      }).map((_, idx) => (
                        <button
                          key={idx}
                          onClick={() => {
                            setIsPlaying(false);
                            if (weaponSet === "draft") {
                              setDraftFrameIndex(idx);
                            } else {
                              setLiveFrameIndex(idx);
                            }
                          }}
                          className={`flex-1 py-1 rounded text-xs font-mono font-bold transition-all cursor-pointer ${
                            (weaponSet === "draft" ? draftFrameIndex : liveFrameIndex) === idx
                              ? "bg-amber-500 text-black shadow-md shadow-amber-500/20"
                              : "bg-slate-800/80 hover:bg-slate-700 text-slate-300"
                          }`}
                        >
                          F{idx + 1}
                        </button>
                      ))}
                    </div>
                  </div>

                  <div className="text-xs text-slate-300 font-mono bg-black/50 border border-slate-800/80 rounded p-2 flex items-start gap-2">
                    <Info className="w-3.5 h-3.5 text-amber-400 shrink-0 mt-0.5" />
                    <div>
                      {weaponSet === "draft" ? (
                        <span>
                          <strong className="text-amber-300">
                            {currentDraftConfig.label} (Frame {draftFrameIndex + 1} of {currentDraftConfig.frames}):
                          </strong>{" "}
                          {currentDraftConfig.description}
                        </span>
                      ) : (
                        <span>
                          <strong className="text-cyan-300">
                            Live Engine Frame ({liveMode} F{liveFrameIndex + 1}):
                          </strong>{" "}
                          Rendered from {selectedSpec.sourceFiles[liveMode === "aim" ? "idle" : liveMode === "empty" ? "idle" : liveMode]}.
                        </span>
                      )}
                    </div>
                  </div>
                </div>
              </div>

              {/* 5x5 Full Grid Sheet Overview (if toggled) */}
              {weaponSet === "draft" && showGridOverview && (
                <div className="bg-[#0b1016] border border-amber-500/40 rounded-lg p-4 space-y-3">
                  <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                    <h3 className="text-xs font-bold text-amber-400 font-mono uppercase tracking-wider flex items-center gap-1.5">
                      <Grid className="w-4 h-4" />
                      5×5 (25-FRAME) MASTER SPRITE SHEET OVERVIEW
                    </h3>
                    <span className="text-[10px] text-slate-400 font-mono">
                      2560×1920 RGBA · 512×384 per cell
                    </span>
                  </div>

                  <div className="relative border border-slate-800 rounded overflow-hidden max-h-[400px] overflow-y-auto">
                    <img
                      src={asset(selectedDraft.sheet_file)}
                      alt="5x5 Master Sheet"
                      className="w-full object-contain"
                    />
                  </div>
                </div>
              )}

              {/* Source Assets & Hashes Card */}
              <div className="bg-[#0b1016] border border-slate-800 rounded-lg p-4">
                <div className="flex items-center justify-between mb-3 border-b border-slate-800 pb-2">
                  <h3 className="text-xs font-bold text-amber-400 font-mono uppercase tracking-wider flex items-center gap-1.5">
                    <Hash className="w-4 h-4" />
                    {weaponSet === "draft" ? "DRAFT ASSET HASHES" : "LIVE ENGINE ASSET HASHES"}
                  </h3>
                  <span className="text-[10px] text-slate-400 font-mono">
                    Click copy icon to reference exact hash in chat
                  </span>
                </div>

                {weaponSet === "draft" ? (
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-3 font-mono text-xs">
                    {/* 1. 5x5 Sheet */}
                    <div className="bg-black/50 border border-slate-800/80 rounded p-3 flex flex-col justify-between gap-2">
                      <div>
                        <div className="text-slate-400 text-[10px] uppercase">5×5 Master Sprite Sheet (25 Frames)</div>
                        <div className="font-bold text-white text-xs mt-0.5 truncate">{selectedDraft.sheet_file}</div>
                      </div>
                      <div className="pt-2 border-t border-slate-800/60 flex items-center justify-between">
                        <span className="text-amber-400 text-[11px]">#{selectedDraft.sheet_short_hash}</span>
                        <button
                          onClick={() => copyToClipboard(selectedDraft.sheet_hash, selectedDraft.sheet_short_hash)}
                          className="p-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 transition-all cursor-pointer"
                          title="Copy Full SHA-256"
                        >
                          {copiedHash === selectedDraft.sheet_short_hash ? (
                            <Check className="w-3.5 h-3.5 text-green-400" />
                          ) : (
                            <Copy className="w-3.5 h-3.5" />
                          )}
                        </button>
                      </div>
                    </div>

                    {/* 2. 1-Frame Aim Ref */}
                    <div className="bg-black/50 border border-slate-800/80 rounded p-3 flex flex-col justify-between gap-2">
                      <div>
                        <div className="text-slate-400 text-[10px] uppercase">1-Frame Aim Point Reference</div>
                        <div className="font-bold text-white text-xs mt-0.5 truncate">{selectedDraft.aim_file}</div>
                      </div>
                      <div className="pt-2 border-t border-slate-800/60 flex items-center justify-between">
                        <span className="text-amber-400 text-[11px]">#{selectedDraft.aim_short_hash}</span>
                        <button
                          onClick={() => copyToClipboard(selectedDraft.aim_hash, selectedDraft.aim_short_hash)}
                          className="p-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 transition-all cursor-pointer"
                          title="Copy Full SHA-256"
                        >
                          {copiedHash === selectedDraft.aim_short_hash ? (
                            <Check className="w-3.5 h-3.5 text-green-400" />
                          ) : (
                            <Copy className="w-3.5 h-3.5" />
                          )}
                        </button>
                      </div>
                    </div>

                    {/* 3. AI Master Render */}
                    <div className="bg-black/50 border border-slate-800/80 rounded p-3 flex flex-col justify-between gap-2">
                      <div className="flex items-start gap-2">
                        {selectedDraft.gen_master_file && (
                          <img
                            src={asset(selectedDraft.gen_master_file)}
                            alt="AI Master"
                            className="w-10 h-10 object-cover rounded border border-slate-700/80 flex-shrink-0"
                          />
                        )}
                        <div className="min-w-0 flex-1">
                          <div className="text-slate-400 text-[10px] uppercase">AI Master Render (Source)</div>
                          <div className="font-bold text-white text-xs mt-0.5 truncate">
                            {selectedDraft.gen_master_file || "N/A"}
                          </div>
                        </div>
                      </div>
                      <div className="pt-2 border-t border-slate-800/60 flex items-center justify-between">
                        <span className="text-cyan-400 text-[11px]">
                          #{selectedDraft.gen_master_short_hash || selectedDraft.sheet_short_hash}
                        </span>
                        {selectedDraft.gen_master_hash && (
                          <button
                            onClick={() =>
                              copyToClipboard(
                                selectedDraft.gen_master_hash!,
                                selectedDraft.gen_master_short_hash!
                              )
                            }
                            className="p-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 transition-all cursor-pointer"
                            title="Copy Full SHA-256"
                          >
                            {copiedHash === selectedDraft.gen_master_short_hash ? (
                              <Check className="w-3.5 h-3.5 text-green-400" />
                            ) : (
                              <Copy className="w-3.5 h-3.5" />
                            )}
                          </button>
                        )}
                      </div>
                    </div>
                  </div>
                ) : (
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-3 font-mono text-xs">
                    {/* Live Idle */}
                    {(() => {
                      const item = getAsset(selectedSpec.sourceFiles.idle);
                      return (
                        <div className="bg-black/50 border border-slate-800/80 rounded p-2.5 flex flex-col justify-between gap-2">
                          <div>
                            <div className="text-slate-400 text-[10px] uppercase">Live Idle / Aim</div>
                            <div className="font-bold text-white text-xs mt-0.5 truncate">{selectedSpec.sourceFiles.idle}</div>
                          </div>
                          {item && (
                            <div className="pt-2 border-t border-slate-800/60 flex items-center justify-between">
                              <span className="text-cyan-400 text-[11px]">#{item.shortHash}</span>
                              <button
                                onClick={() => copyToClipboard(item.hash, item.shortHash)}
                                className="p-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 cursor-pointer"
                              >
                                {copiedHash === item.shortHash ? (
                                  <Check className="w-3.5 h-3.5 text-green-400" />
                                ) : (
                                  <Copy className="w-3.5 h-3.5" />
                                )}
                              </button>
                            </div>
                          )}
                        </div>
                      );
                    })()}

                    {/* Live Fire */}
                    {(() => {
                      const item = getAsset(selectedSpec.sourceFiles.fire);
                      return (
                        <div className="bg-black/50 border border-slate-800/80 rounded p-2.5 flex flex-col justify-between gap-2">
                          <div>
                            <div className="text-slate-400 text-[10px] uppercase">Live Firing Sheet</div>
                            <div className="font-bold text-white text-xs mt-0.5 truncate">{selectedSpec.sourceFiles.fire}</div>
                          </div>
                          {item && (
                            <div className="pt-2 border-t border-slate-800/60 flex items-center justify-between">
                              <span className="text-cyan-400 text-[11px]">#{item.shortHash}</span>
                              <button
                                onClick={() => copyToClipboard(item.hash, item.shortHash)}
                                className="p-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 cursor-pointer"
                              >
                                {copiedHash === item.shortHash ? (
                                  <Check className="w-3.5 h-3.5 text-green-400" />
                                ) : (
                                  <Copy className="w-3.5 h-3.5" />
                                )}
                              </button>
                            </div>
                          )}
                        </div>
                      );
                    })()}

                    {/* Live Reload */}
                    {(() => {
                      const item = getAsset(selectedSpec.sourceFiles.reload);
                      return (
                        <div className="bg-black/50 border border-slate-800/80 rounded p-2.5 flex flex-col justify-between gap-2">
                          <div>
                            <div className="text-slate-400 text-[10px] uppercase">Live Reload Sheet</div>
                            <div className="font-bold text-white text-xs mt-0.5 truncate">{selectedSpec.sourceFiles.reload}</div>
                          </div>
                          {item && (
                            <div className="pt-2 border-t border-slate-800/60 flex items-center justify-between">
                              <span className="text-cyan-400 text-[11px]">#{item.shortHash}</span>
                              <button
                                onClick={() => copyToClipboard(item.hash, item.shortHash)}
                                className="p-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 cursor-pointer"
                              >
                                {copiedHash === item.shortHash ? (
                                  <Check className="w-3.5 h-3.5 text-green-400" />
                                ) : (
                                  <Copy className="w-3.5 h-3.5" />
                                )}
                              </button>
                            </div>
                          )}
                        </div>
                      );
                    })()}
                  </div>
                )}
              </div>
            </div>
          </div>
        ) : (
          /* General Grid for other categories */
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h2 className="text-sm font-bold text-amber-400 font-mono uppercase tracking-wider">
                {activeTab.toUpperCase()} ARTIFACTS ({filteredAssets.length})
              </h2>
              <span className="text-xs text-slate-400 font-mono">
                Click any hash to copy for feedback
              </span>
            </div>

            <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-3">
              {filteredAssets.map((assetItem) => {
                const isImage =
                  assetItem.name.endsWith(".png") ||
                  assetItem.name.endsWith(".jpg") ||
                  assetItem.name.endsWith(".svg");
                const isAudio =
                  assetItem.name.endsWith(".ogg") ||
                  assetItem.name.endsWith(".mp3") ||
                  assetItem.name.endsWith(".wav");

                return (
                  <div
                    key={assetItem.file}
                    className="bg-[#0b1016] border border-slate-800/80 rounded p-2.5 flex flex-col justify-between gap-2 hover:border-slate-700 transition-all group"
                  >
                    <div className="h-28 bg-black/60 rounded border border-slate-900 flex items-center justify-center p-1 overflow-hidden relative">
                      {"group" in assetItem && assetItem.group === "textures" && !("seamless" in assetItem && assetItem.seamless === false) ? (
                        <div
                          className="w-full h-full rounded"
                          style={{ backgroundImage: `url(${asset(assetItem.file)})`, backgroundRepeat: "repeat", backgroundSize: "50% 50%" }}
                          title="2×2 seamless tile preview"
                        />
                      ) : isImage ? (
                        <img
                          src={asset(assetItem.file)}
                          alt={assetItem.name}
                          className="max-h-full max-w-full object-contain group-hover:scale-105 transition-transform"
                        />
                      ) : isAudio ? (
                        <div className="flex flex-col items-center gap-1 text-slate-400">
                          <Volume2 className="w-6 h-6 text-amber-500/70" />
                          <audio
                            controls
                            src={asset(assetItem.file)}
                            className="w-24 h-6 opacity-60 hover:opacity-100"
                          />
                        </div>
                      ) : (
                        <Package className="w-6 h-6 text-slate-600" />
                      )}

                      {assetItem.width && assetItem.height && (
                        <span className="absolute bottom-1 right-1 text-[8px] font-mono px-1 rounded bg-black/80 text-slate-400">
                          {assetItem.width}×{assetItem.height}
                        </span>
                      )}
                      {"group" in assetItem && assetItem.group === "textures" && !("seamless" in assetItem && assetItem.seamless === false) && (
                        <span className="absolute top-1 left-1 text-[8px] font-mono px-1 rounded bg-black/80 text-amber-300">2×2 TILE</span>
                      )}
                    </div>

                    <div>
                      <div className="font-mono text-xs text-slate-200 truncate font-bold" title={assetItem.name}>
                        {assetItem.name}
                      </div>
                      <div className="text-[10px] font-mono text-slate-400">
                        {(assetItem.size / 1024).toFixed(0)} KB
                      </div>
                    </div>

                    <div className="pt-1.5 border-t border-slate-800/60 flex items-center justify-between font-mono text-xs">
                      <span className="text-amber-400 text-[10px]">#{assetItem.shortHash}</span>
                      <button
                        onClick={() => copyToClipboard(assetItem.hash, assetItem.shortHash)}
                        className="p-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 transition-all cursor-pointer"
                        title="Copy SHA-256 hash"
                      >
                        {copiedHash === assetItem.shortHash ? (
                          <Check className="w-3 h-3 text-green-400" />
                        ) : (
                          <Copy className="w-3 h-3" />
                        )}
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </main>
    </div>
  );
}
