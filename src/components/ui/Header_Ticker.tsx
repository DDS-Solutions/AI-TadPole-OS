import React from 'react';

interface HeaderTickerProps {
    children: React.ReactNode;
    duration?: number; // in seconds
}

/**
 * Header_Ticker
 * A premium infinite scrolling ticker for the OS header.
 * Uses CSS marquee for smooth 60fps performance and low CPU overhead.
 */
export const Header_Ticker: React.FC<HeaderTickerProps> = ({ 
    children, 
    duration = 40 
}) => {
    return (
        <div className="relative flex items-center overflow-hidden w-full h-full pause-on-hover">
            {/* The gradient masks for a smooth fade-in/out effect */}
            <div className="absolute inset-y-0 left-0 w-12 bg-gradient-to-r from-zinc-950 to-transparent z-10 pointer-events-none" />
            <div className="absolute inset-y-0 right-0 w-12 bg-gradient-to-l from-zinc-950 to-transparent z-10 pointer-events-none" />

            <div 
                className="flex items-center gap-16 animate-marquee whitespace-nowrap px-4"
                style={{ '--duration': `${duration}s` } as React.CSSProperties}
            >
                {/* Original Items */}
                <div className="flex items-center gap-16">
                    {children}
                </div>
                {/* Duplicated Items for seamless loop */}
                <div className="flex items-center gap-16" aria-hidden="true">
                    {children}
                </div>
            </div>
        </div>
    );
};
