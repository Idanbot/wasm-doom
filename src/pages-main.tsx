import { createRoot } from "react-dom/client";
import { GameApp } from "@/components/game-app";
import "./styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("Missing #root");
createRoot(root).render(<GameApp />);
