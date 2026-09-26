import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { AssetCatalog } from "@/components/catalog/AssetCatalog";
import { isCatalogEnabled } from "@/lib/catalog-guard";

export const Route = createFileRoute("/catalog")({ component: CatalogPage });

function CatalogPage() {
  const navigate = useNavigate();
  const [mounted, setMounted] = useState(false);

  useEffect(() => setMounted(true), []);

  useEffect(() => {
    if (mounted && !isCatalogEnabled()) {
      navigate({ to: "/" });
    }
  }, [mounted, navigate]);

  if (!mounted) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-[#080d11] text-amber-400 font-mono text-sm" role="status">
        Preparing BLACKSITE asset catalog…
      </div>
    );
  }

  if (!isCatalogEnabled()) {
    return (
      <div className="flex h-screen w-screen items-center justify-center bg-[#080d11] text-amber-500 font-mono text-sm">
        [BLACKSITE] Access denied: Asset Catalog is restricted to local development environments.
      </div>
    );
  }

  return <AssetCatalog onBackToGame={() => window.location.assign("/")} />;
}
