"use client";

import { useEffect, useState } from "react";

export interface BlendPoolApyData {
  usdcSupplyApy: number | null;
  xlmSupplyApy: number | null;
  isLoading: boolean;
  error: string | null;
}

const BLEND_USDC_POOL_ID = process.env.NEXT_PUBLIC_BLEND_USDC_POOL_ID ?? "";
const RPC_URL = process.env.NEXT_PUBLIC_STELLAR_RPC_URL ?? "https://soroban-testnet.stellar.org";
const NETWORK_PASSPHRASE = process.env.NEXT_PUBLIC_STELLAR_NETWORK ?? "Test SDF Network ; September 2015";

const CACHE_TTL_MS = 5 * 60 * 1000; // 5 minutes

let cachedData: BlendPoolApyData | null = null;
let cacheTimestamp = 0;

/**
 * Fetches live APY data from Blend Protocol's on-chain lending pools.
 * Falls back to null values if Blend is unavailable or the pool ID is not configured.
 *
 * Usage: pool reserves are read via the @blend-capital/blend-sdk Pool.load() call.
 * The resulting supply_apy fields reflect the current annualised rate for USDC and XLM.
 */
export function useBlendApy(): BlendPoolApyData {
  const [data, setData] = useState<BlendPoolApyData>({
    usdcSupplyApy: null,
    xlmSupplyApy: null,
    isLoading: false,
    error: null,
  });

  useEffect(() => {
    if (!BLEND_USDC_POOL_ID) return;

    // Return cached data if it is still fresh
    if (cachedData && Date.now() - cacheTimestamp < CACHE_TTL_MS) {
      setData(cachedData);
      return;
    }

    let cancelled = false;

    const fetchApy = async () => {
      setData((prev) => ({ ...prev, isLoading: true, error: null }));

      try {
        const { PoolV2 } = await import("@blend-capital/blend-sdk");

        const network = {
          rpc: RPC_URL,
          passphrase: NETWORK_PASSPHRASE,
          opts: { allowHttp: true },
        };

        const pool = await PoolV2.load(network, BLEND_USDC_POOL_ID);

        let usdcSupplyApy: number | null = null;
        let xlmSupplyApy: number | null = null;

        const usdcId = process.env.NEXT_PUBLIC_BLEND_USDC_CONTRACT ?? "";
        const xlmId = process.env.NEXT_PUBLIC_BLEND_XLM_CONTRACT ?? "";

        // reserves is a Map<string, Reserve> where key is the asset contract ID
        pool.reserves.forEach((reserve, assetId) => {
          if (assetId === usdcId && reserve.estSupplyApy != null) {
            usdcSupplyApy = reserve.estSupplyApy * 100;
          }
          if (assetId === xlmId && reserve.estSupplyApy != null) {
            xlmSupplyApy = reserve.estSupplyApy * 100;
          }
        });

        if (!cancelled) {
          const result: BlendPoolApyData = {
            usdcSupplyApy,
            xlmSupplyApy,
            isLoading: false,
            error: null,
          };
          cachedData = result;
          cacheTimestamp = Date.now();
          setData(result);
        }
      } catch (err) {
        if (!cancelled) {
          setData({
            usdcSupplyApy: null,
            xlmSupplyApy: null,
            isLoading: false,
            error: err instanceof Error ? err.message : "Failed to fetch Blend APY",
          });
        }
      }
    };

    fetchApy();
    return () => { cancelled = true; };
  }, []);

  return data;
}
