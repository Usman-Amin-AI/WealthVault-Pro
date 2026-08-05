import { PERFORMANCE_CHART_COLORS } from "@/components/performance-chart-colors";
import { TradingViewChart, TradingViewSeries } from "@/components/tradingview-chart";
import {
  calculateBollingerBands,
  calculateMACD,
  calculateRSI,
  normalizeReturnsForChart,
} from "@/lib/indicators";
import { ReturnData } from "@/lib/types";
import { useMemo, useState } from "react";
import { Button } from "@investwise/ui";

interface PerformanceChartProps {
  data: {
    id: string;
    name: string;
    returns: ReturnData[];
  }[];
}

export function PerformanceChart({ data }: PerformanceChartProps) {
  const [showMACD, setShowMACD] = useState(false);
  const [showRSI, setShowRSI] = useState(false);
  const [showBB, setShowBB] = useState(false);

  // Memoize the series formatting
  const series: TradingViewSeries[] = useMemo(() => {
    return data.map((d, index) => ({
      id: d.id,
      name: d.name,
      color: PERFORMANCE_CHART_COLORS[index % PERFORMANCE_CHART_COLORS.length],
      data: normalizeReturnsForChart(d.returns),
    }));
  }, [data]);

  // Compute indicators for the primary series (data[0]) if enabled
  const indicators = useMemo(() => {
    const primarySeries = data[0]?.returns || [];
    return {
      macd: showMACD ? calculateMACD(primarySeries) : undefined,
      rsi: showRSI ? calculateRSI(primarySeries) : undefined,
      bollingerBands: showBB ? calculateBollingerBands(primarySeries) : undefined,
    };
  }, [data, showMACD, showRSI, showBB]);

  const activeIndicators = {
    macd: showMACD,
    rsi: showRSI,
    bollingerBands: showBB,
  };

  return (
    <div className="flex h-full w-full flex-col relative">
      {/* Absolute positioned toolbar inside the chart area, top right or top left */}
      <div className="absolute top-2 left-2 z-10 flex gap-2">
        <Button
          variant={showBB ? "default" : "outline"}
          size="sm"
          className="h-7 text-xs font-medium opacity-80 hover:opacity-100"
          onClick={() => setShowBB(!showBB)}
        >
          BB
        </Button>
        <Button
          variant={showMACD ? "default" : "outline"}
          size="sm"
          className="h-7 text-xs font-medium opacity-80 hover:opacity-100"
          onClick={() => setShowMACD(!showMACD)}
        >
          MACD
        </Button>
        <Button
          variant={showRSI ? "default" : "outline"}
          size="sm"
          className="h-7 text-xs font-medium opacity-80 hover:opacity-100"
          onClick={() => setShowRSI(!showRSI)}
        >
          RSI
        </Button>
      </div>

      <div className="min-h-0 flex-1">
        <TradingViewChart
          series={series}
          indicators={indicators}
          activeIndicators={activeIndicators}
        />
      </div>
    </div>
  );
}
