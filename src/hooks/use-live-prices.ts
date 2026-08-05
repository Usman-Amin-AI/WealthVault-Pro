import { useEffect, useState, useRef } from "react";
import { LivePrice, startLiveStream, getLivePrices } from "@/commands/market-data";
import { listenLivePriceUpdate } from "@/adapters";

export function useLivePrices(symbols: string[], provider: string = "YAHOO") {
  const [livePrices, setLivePrices] = useState<Record<string, LivePrice>>({});
  const symbolsRef = useRef(symbols);
  const providerRef = useRef(provider);

  useEffect(() => {
    symbolsRef.current = symbols;
    providerRef.current = provider;
  }, [symbols, provider]);

  useEffect(() => {
    if (symbols.length === 0) return;

    let isMounted = true;
    let unlisten: (() => void) | undefined;

    const initLiveStream = async () => {
      try {
        // First get the latest prices from cache immediately
        const initialPrices = await getLivePrices(symbols);
        if (isMounted) {
          setLivePrices((prev) => ({ ...prev, ...initialPrices }));
        }

        // Start listening to the broadcast events
        unlisten = await listenLivePriceUpdate((event: any) => {
          if (!isMounted) return;
          const { payload } = event;

          if (payload && payload.symbol && payload.price) {
            setLivePrices((prev) => ({
              ...prev,
              [payload.symbol]: {
                symbol: payload.symbol,
                price: payload.price,
                timestamp: payload.timestamp,
              },
            }));
          }
        });

        // Trigger the backend to start the stream
        await startLiveStream(symbols, provider);
      } catch (err) {
        console.error("Failed to initialize live stream:", err);
      }
    };

    initLiveStream();

    return () => {
      isMounted = false;
      if (unlisten) unlisten();
    };
  }, [symbols.join(","), provider]);

  return { livePrices };
}
