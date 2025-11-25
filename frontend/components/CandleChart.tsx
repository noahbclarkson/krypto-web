"use client";

import {
    createChart,
    ColorType,
    IChartApi,
    ISeriesApi,
    CandlestickSeries,
    CandlestickData,
    Time,
    UTCTimestamp,
} from 'lightweight-charts';
import { useEffect, useRef } from 'react';

type CandleData = Omit<CandlestickData<Time>, 'time'> & {
    time: string | number | UTCTimestamp;
};

interface Props {
    data: CandleData[];
    colors?: {
        backgroundColor?: string;
        lineColor?: string;
        textColor?: string;
        areaTopColor?: string;
        areaBottomColor?: string;
    };
}

export const CandleChart = ({ data, colors = {} }: Props) => {
    const chartContainerRef = useRef<HTMLDivElement>(null);
    const chartRef = useRef<IChartApi | null>(null);
    const seriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
    const hasFitRef = useRef(false);

    // Init chart once
    useEffect(() => {
        if (!chartContainerRef.current) return;

        const rect = chartContainerRef.current.getBoundingClientRect();
        const width = rect.width || chartContainerRef.current.clientWidth;
        const height = rect.height > 40
            ? rect.height
            : (chartContainerRef.current.clientHeight || 250);

        const chart = createChart(chartContainerRef.current, {
            layout: {
                background: { type: ColorType.Solid, color: colors.backgroundColor || '#020617' },
                textColor: colors.textColor || '#94a3b8',
            },
            width,
            height,
            grid: {
                vertLines: { color: '#1e293b' },
                horzLines: { color: '#1e293b' },
            },
            timeScale: {
                timeVisible: true,
                secondsVisible: false,
            }
        });

        chartRef.current = chart;

        const candlestickSeries = chart.addSeries(CandlestickSeries, {
            upColor: '#22c55e',
            downColor: '#ef4444',
            borderVisible: false,
            wickUpColor: '#22c55e',
            wickDownColor: '#ef4444',
        });
        seriesRef.current = candlestickSeries;

        const handleResize = () => {
            if (!chartContainerRef.current || !chartRef.current) return;
            const r = chartContainerRef.current.getBoundingClientRect();
            const w = r.width || chartContainerRef.current.clientWidth;
            const h = r.height > 40
                ? r.height
                : (chartContainerRef.current.clientHeight || 250);
            chartRef.current.applyOptions({
                width: w,
                height: h,
            });
        };

        window.addEventListener('resize', handleResize);

        return () => {
            window.removeEventListener('resize', handleResize);
            chart.remove();
            chartRef.current = null;
            seriesRef.current = null;
            hasFitRef.current = false;
        };
    }, []);

    // Apply theme changes
    useEffect(() => {
        if (!chartRef.current) return;
        chartRef.current.applyOptions({
            layout: {
                background: { type: ColorType.Solid, color: colors.backgroundColor || '#020617' },
                textColor: colors.textColor || '#94a3b8',
            },
        });
        seriesRef.current?.applyOptions({
            upColor: '#22c55e',
            downColor: '#ef4444',
            borderVisible: false,
            wickUpColor: '#22c55e',
            wickDownColor: '#ef4444',
        });
    }, [colors.backgroundColor, colors.textColor]);

    // Stream new data without resetting zoom
    useEffect(() => {
        if (!seriesRef.current || !chartRef.current) return;

        const formattedData: CandlestickData<Time>[] = data.map(d => ({
            ...d,
            time: (typeof d.time === 'string'
                ? (Math.floor(new Date(d.time).getTime() / 1000) as UTCTimestamp)
                : (d.time as UTCTimestamp)),
        }));

        seriesRef.current.setData(formattedData);

        if (!hasFitRef.current && formattedData.length > 0) {
            chartRef.current.timeScale().fitContent();
            hasFitRef.current = true;
        }
    }, [data]);

    return (
        <div ref={chartContainerRef} className="w-full h-full relative" />
    );
};
