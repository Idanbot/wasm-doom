import { execFileSync } from "node:child_process";

/** Embed the source update date in both shells; never use the visitor's clock. */
export function latestUpdateDate(root = process.cwd()): string {
  try {
    const date = execFileSync("git", ["log", "-1", "--format=%cs"], {
      cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"], timeout: 5000,
    }).trim();
    if (/^\d{4}-\d{2}-\d{2}$/.test(date)) return date;
  } catch { /* Source archives have no Git metadata; use their build date. */ }
  return new Date().toISOString().slice(0, 10);
}
