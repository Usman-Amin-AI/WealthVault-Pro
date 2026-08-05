import { ReturnData } from "@/lib/types";
import { BollingerBands, MACD, RSI } from "technicalindicators";

export interface ChartPoint {
  time: string;
  value: number;
}

function toChartDate(dateString: string): string | null {
  if (!dateString) return null;

  const parsedDate = new Date(dateString);
  if (Number.isNaN(parsedDate.getTime())) {
    return null;
  }

  return parsedDate.toISOString().split("T")[0] ?? null;
}

export interface MACDResult {
  time: string;
  macd: number;
  signal: number;
  histogram: number;
}

export interface BBResult {
  time: string;
  upper: number;
  middle: number;
  lower: number;
}

export interface RSIResult {
  time: string;
  value: number;
}

/**
 * Normalizes returns to a format lightweight-charts expects (YYYY-MM-DD or Unix timestamp).
 * Since dates might be ISO strings, we extract YYYY-MM-DD.
 */
export function normalizeReturnsForChart(returns: ReturnData[]): ChartPoint[] {
  const normalizedByTime = new Map<string, number>();

  returns.forEach((r) => {
    const time = toChartDate(r.date);
    if (!time) return;

    normalizedByTime.set(time, r.value);
  });

  return Array.from(normalizedByTime.entries())
    .sort((a, b) => a[0].localeCompare(b[0]))
    .map(([time, value]) => ({ time, value }));
}

/**
 * Calculates MACD (12, 26, 9)
 */
export function calculateMACD(returns: ReturnData[]): MACDResult[] {
  if (returns.length < 26) return []; // Not enough data

  const values = returns.map((r) => r.value);
  const macdInput = {
    values: values,
    fastPeriod: 12,
    slowPeriod: 26,
    signalPeriod: 9,
    SimpleMAOscillator: false,
    SimpleMASignal: false,
  };

  const macdResult = MACD.calculate(macdInput);

  // MACD results are shorter than the input array. We need to align them.
  const padding = returns.length - macdResult.length;

  return macdResult.flatMap((result, i) => {
    const originalReturn = returns[i + padding];
    const time = toChartDate(originalReturn.date);
    if (!time) return [];

    return [
      {
        time,
        macd: result.MACD || 0,
        signal: result.signal || 0,
        histogram: result.histogram || 0,
      },
    ];
  });
}

/**
 * Calculates RSI (14)
 */
export function calculateRSI(returns: ReturnData[]): RSIResult[] {
  const period = 14;
  if (returns.length < period) return [];

  const values = returns.map((r) => r.value);
  const rsiInput = {
    values: values,
    period: period,
  };

  const rsiResult = RSI.calculate(rsiInput);
  const padding = returns.length - rsiResult.length;

  return rsiResult.flatMap((val, i) => {
    const originalReturn = returns[i + padding];
    const time = toChartDate(originalReturn.date);
    if (!time) return [];

    return [{ time, value: val }];
  });
}

/**
 * Calculates Bollinger Bands (20, 2)
 */
export function calculateBollingerBands(returns: ReturnData[]): BBResult[] {
  const period = 20;
  if (returns.length < period) return [];

  const values = returns.map((r) => r.value);
  const bbInput = {
    period: period,
    values: values,
    stdDev: 2,
  };

  const bbResult = BollingerBands.calculate(bbInput);
  const padding = returns.length - bbResult.length;

  return bbResult.flatMap((res, i) => {
    const originalReturn = returns[i + padding];
    const time = toChartDate(originalReturn.date);
    if (!time) return [];

    return [
      {
        time,
        upper: res.upper,
        middle: res.middle,
        lower: res.lower,
      },
    ];
  });
}
