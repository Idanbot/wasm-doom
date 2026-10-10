export function verifiedCompletion(episode) {
  return (
    episode.status === "COMPLETE" &&
    episode.completed === true &&
    episode.deterministicReplay === true
  );
}

export function completionStatus(episodes) {
  if (episodes.length && episodes.every(verifiedCompletion)) return "MET";
  if (episodes.some((episode) => episode.status === "SYSTEM_FAILURE")) return "NOT_MET";
  return episodes.some((episode) => episode.status === "INCONCLUSIVE") ? "INCONCLUSIVE" : "NOT_MET";
}
