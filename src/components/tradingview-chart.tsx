import {
  CandlestickSeries,
  ColorType,
  CrosshairMode,
  HistogramSeries,
  IChartApi,
  ISeriesApi,
  LineSeries,
  SeriesType,
  createChart,
} from "lightweight-charts";
import { useEffect, useRef, useState } from "react";

export interface TradingViewCandle {
  time: string;
  open: number;
  high: number;
  low: number;
  close: number;
}

export interface TradingViewSeries {
  id: string;
  name: string;
  color: string;
  type?: "line" | "candlestick";
  data: { time: string; value: number }[];
  candleData?: TradingViewCandle[];
}

export interface IndicatorConfig {
  macd: boolean;
  rsi: boolean;
  bollingerBands: boolean;
}

export interface TradingViewChartProps {
  series: TradingViewSeries[];
  indicators?: {
    macd?: { time: string; macd: number; signal: number; histogram: number }[];
    rsi?: { time: string; value: number }[];
    bollingerBands?: { time: string; upper: number; middle: number; lower: number }[];
  };
  activeIndicators?: IndicatorConfig;
  className?: string;
}

export function TradingViewChart({
  series,
  indicators,
  activeIndicators,
  className,
}: TradingViewChartProps) {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRefs = useRef<Map<string, ISeriesApi<SeriesType>>>(new Map());
  const indicatorRefs = useRef<Map<string, ISeriesApi<SeriesType>>>(new Map());
  const [isDark, setIsDark] = useState(false);

  useEffect(() => {
    if (typeof window === "undefined") return;

    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

    const update = () => setIsDark(mediaQuery.matches);
    update();

    mediaQuery.addEventListener?.("change", update);
    mediaQuery.addListener?.(update);

    return () => {
      mediaQuery.removeEventListener?.("change", update);
      mediaQuery.removeListener?.(update);
    };
  }, []);

  useEffect(() => {
    if (!chartContainerRef.current) return;

    // Chart configuration matching our previous recharts aesthetic
    const chart = createChart(chartContainerRef.current, {
      layout: {
        background: { type: ColorType.Solid, color: "transparent" },
        textColor: isDark ? "#A3A3A3" : "#52525B", // muted-foreground
      },
      grid: {
        vertLines: { visible: false },
        horzLines: {
          color: isDark ? "#27272A" : "#E4E4E7", // border color
          style: 1, // Dotted
        },
      },
      crosshair: {
        mode: CrosshairMode.Magnet,
      },
      rightPriceScale: {
        borderVisible: false,
        alignLabels: true,
      },
      timeScale: {
        borderVisible: false,
        timeVisible: true,
        fixLeftEdge: true,
        fixRightEdge: true,
      },
      handleScroll: {
        mouseWheel: true,
        pressedMouseMove: true,
      },
      handleScale: {
        mouseWheel: true,
        axisPressedMouseMove: true,
      },
    });

    chartRef.current = chart;

    // Handle resize
    const handleResize = () => {
      if (chartContainerRef.current) {
        chart.applyOptions({
          width: chartContainerRef.current.clientWidth,
          height: chartContainerRef.current.clientHeight,
        });
      }
    };

    window.addEventListener("resize", handleResize);

    // Initial resize to fit container
    handleResize();

    return () => {
      window.removeEventListener("resize", handleResize);
      chart.remove();
      chartRef.current = null;
      seriesRefs.current.clear();
      indicatorRefs.current.clear();
    };
  }, [isDark]);

  // Update Data and Indicators
  useEffect(() => {
    if (!chartRef.current) return;

    const chart = chartRef.current;

    // 1. Manage main series lines and candles
    const currentSeriesIds = new Set(series.map((s) => s.id));

    // Remove deleted series
    for (const [id, seriesApi] of Array.from(seriesRefs.current.entries())) {
      if (!currentSeriesIds.has(id)) {
        chart.removeSeries(seriesApi);
        seriesRefs.current.delete(id);
      }
    }

    // Add or update existing series
    series.forEach((s) => {
      const existingSeries = seriesRefs.current.get(s.id);
      if (existingSeries) {
        chart.removeSeries(existingSeries);
        seriesRefs.current.delete(s.id);
      }

      const isCandlestickSeries = s.type === "candlestick" && s.candleData?.length;
      const nextSeries = isCandlestickSeries
        ? chart.addSeries(CandlestickSeries, {
            upColor: s.color,
            downColor: "#EF4444",
            wickUpColor: s.color,
            wickDownColor: "#EF4444",
            borderUpColor: s.color,
            borderDownColor: "#EF4444",
          })
        : chart.addSeries(LineSeries, {
            color: s.color,
            lineWidth: 2,
            crosshairMarkerVisible: true,
            crosshairMarkerRadius: 4,
            crosshairMarkerBorderColor: isDark ? "#18181B" : "#FFFFFF",
            crosshairMarkerBackgroundColor: s.color,
          });

      seriesRefs.current.set(s.id, nextSeries as ISeriesApi<SeriesType>);

      const uniqueData = Array.from(
        new Map(
          (isCandlestickSeries ? (s.candleData ?? []) : s.data).map((item) => [item.time, item]),
        ).values(),
      ).sort((a, b) => a.time.localeCompare(b.time));

      nextSeries.setData(uniqueData as never);
    });

    // 2. Manage Indicators

    // Clear all existing indicator series first
    for (const api of Array.from(indicatorRefs.current.values())) {
      chart.removeSeries(api);
    }
    indicatorRefs.current.clear();

    if (activeIndicators?.bollingerBands && indicators?.bollingerBands) {
      const upper = chart.addSeries(LineSeries, {
        color: "rgba(59, 130, 246, 0.4)",
        lineWidth: 1,
        crosshairMarkerVisible: false,
      });
      const middle = chart.addSeries(LineSeries, {
        color: "rgba(59, 130, 246, 0.7)",
        lineWidth: 1,
        lineStyle: 1,
        crosshairMarkerVisible: false,
      });
      const lower = chart.addSeries(LineSeries, {
        color: "rgba(59, 130, 246, 0.4)",
        lineWidth: 1,
        crosshairMarkerVisible: false,
      });

      const uniqueBB = Array.from(
        new Map(indicators.bollingerBands.map((i) => [i.time, i])).values(),
      ).sort((a, b) => a.time.localeCompare(b.time));

      upper.setData(uniqueBB.map((d) => ({ time: d.time, value: d.upper })));
      middle.setData(uniqueBB.map((d) => ({ time: d.time, value: d.middle })));
      lower.setData(uniqueBB.map((d) => ({ time: d.time, value: d.lower })));

      indicatorRefs.current.set("bb_upper", upper as ISeriesApi<SeriesType>);
      indicatorRefs.current.set("bb_middle", middle as ISeriesApi<SeriesType>);
      indicatorRefs.current.set("bb_lower", lower as ISeriesApi<SeriesType>);
    }

    if (activeIndicators?.rsi && indicators?.rsi) {
      // Add RSI as a new pane at the bottom
      const rsiSeries = chart.addSeries(LineSeries, {
        color: "#8B5CF6",
        lineWidth: 2,
        priceScaleId: "rsi",
      });

      chart.priceScale("rsi").applyOptions({
        scaleMargins: {
          top: 0.8,
          bottom: 0,
        },
      });

      const uniqueRsi = Array.from(new Map(indicators.rsi.map((i) => [i.time, i])).values()).sort(
        (a, b) => a.time.localeCompare(b.time),
      );
      rsiSeries.setData(uniqueRsi);
      indicatorRefs.current.set("rsi", rsiSeries);
    }

    if (activeIndicators?.macd && indicators?.macd) {
      // Add MACD as a new pane at the bottom
      const macdSeries = chart.addSeries(LineSeries, {
        color: "#3B82F6",
        lineWidth: 2,
        priceScaleId: "macd",
      });
      const signalSeries = chart.addSeries(LineSeries, {
        color: "#F59E0B",
        lineWidth: 2,
        priceScaleId: "macd",
      });
      const histSeries = chart.addSeries(HistogramSeries, {
        priceScaleId: "macd",
      });

      // Avoid overlapping with RSI if both are enabled
      const topMargin = activeIndicators.rsi ? 0.6 : 0.8;
      const bottomMargin = activeIndicators.rsi ? 0.2 : 0;

      chart.priceScale("macd").applyOptions({
        scaleMargins: {
          top: topMargin,
          bottom: bottomMargin,
        },
      });

      const uniqueMacd = Array.from(new Map(indicators.macd.map((i) => [i.time, i])).values()).sort(
        (a, b) => a.time.localeCompare(b.time),
      );

      macdSeries.setData(uniqueMacd.map((d) => ({ time: d.time, value: d.macd })));
      signalSeries.setData(uniqueMacd.map((d) => ({ time: d.time, value: d.signal })));
      histSeries.setData(
        uniqueMacd.map((d) => ({
          time: d.time,
          value: d.histogram,
          color: d.histogram >= 0 ? "rgba(34, 197, 94, 0.5)" : "rgba(239, 68, 68, 0.5)", // green/red
        })),
      );

      indicatorRefs.current.set("macd_line", macdSeries as ISeriesApi<SeriesType>);
      indicatorRefs.current.set("macd_signal", signalSeries as ISeriesApi<SeriesType>);
      indicatorRefs.current.set("macd_hist", histSeries as ISeriesApi<SeriesType>);
    }

    // Shrink main price scale to leave room for indicators if active
    let mainBottomMargin = 0;
    if (activeIndicators?.rsi) mainBottomMargin += 0.2;
    if (activeIndicators?.macd) mainBottomMargin += 0.2;

    chart.priceScale("right").applyOptions({
      scaleMargins: {
        top: 0.1,
        bottom: mainBottomMargin + 0.05,
      },
    });

    chart.timeScale().fitContent();
  }, [series, indicators, activeIndicators, isDark]);

  return <div ref={chartContainerRef} className={`h-full w-full ${className || ""}`} />;
}
