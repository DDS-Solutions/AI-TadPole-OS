import { useState, useRef, useEffect } from 'react';
import { useMotionValue } from 'framer-motion';
import { useSovereignStore } from '../stores/sovereignStore';

export function useChatWindow() {
    const { isDetached, setDetached } = useSovereignStore();
    const [isMinimized, setIsMinimized] = useState(false);

    const constraintsRef = useRef<HTMLDivElement>(null);

    // Shared drag state for both open and minimized states
    const xOpen = useMotionValue(0);
    const yOpen = useMotionValue(0);
    const xMin = useMotionValue(0);
    const yMin = useMotionValue(0);

    // Prevent open window from spawning off-screen if maximized from a high minimized position
    useEffect(() => {
        if (!isMinimized) {
            const maxNegY = -(window.innerHeight - 600 - 48); // 600px height + 48px combined padding
            const maxNegX = -(window.innerWidth - 400 - 48);  // 400px width + 48px combined padding

            if (yOpen.get() < maxNegY) yOpen.set(Math.min(0, maxNegY));
            if (xOpen.get() < maxNegX) xOpen.set(Math.min(0, maxNegX));

            if (yOpen.get() > 0) yOpen.set(0);
            if (xOpen.get() > 0) xOpen.set(0);
        }
    }, [isMinimized, xOpen, yOpen]);

    const toggleDetach = () => {
        setDetached(!isDetached);
        if (!isDetached) {
            window.open(window.location.href + '#sovereign-chat', 'SovereignChat', 'width=450,height=700');
        }
    };

    const performMinimizeTransform = () => {
        const h = window.innerHeight;
        const w = window.innerWidth;
        const maxNegYOpen = Math.min(-1, -(h - 600 - 48)); // 600px height, 48px combined padding
        const maxNegXOpen = Math.min(-1, -(w - 400 - 48)); // 400px width

        // Safe ratio 0 to 1 depending on where open window sits
        const ratioY = Math.min(1, Math.max(0, yOpen.get() / maxNegYOpen));
        const ratioX = Math.min(1, Math.max(0, xOpen.get() / maxNegXOpen));

        // Scale offsets proportionally
        const yShift = -552 * ratioY;
        const xShift = -180 * ratioX;

        xMin.set(xOpen.get() + xShift);
        yMin.set(yOpen.get() + yShift);
        setIsMinimized(true);
    };

    const performMaximizeTransform = () => {
        const h = window.innerHeight;
        const w = window.innerWidth;
        const maxNegYMin = Math.min(-1, -(h - 48 - 48)); // 48px height approx button
        const maxNegXMin = Math.min(-1, -(w - 220 - 48)); // 220px width approx button

        const ratioY = Math.min(1, Math.max(0, yMin.get() / maxNegYMin));
        const ratioX = Math.min(1, Math.max(0, xMin.get() / maxNegXMin));

        const yShift = 552 * ratioY;
        const xShift = 180 * ratioX;

        yOpen.set(yMin.get() + yShift);
        xOpen.set(xMin.get() + xShift);

        const maxNegYOpen = -(h - 600 - 48);
        const maxNegXOpen = -(w - 400 - 48);

        if (yOpen.get() < maxNegYOpen) yOpen.set(Math.min(0, maxNegYOpen));
        if (xOpen.get() < maxNegXOpen) xOpen.set(Math.min(0, maxNegXOpen));
        if (yOpen.get() > 0) yOpen.set(0);
        if (xOpen.get() > 0) xOpen.set(0);

        setIsMinimized(false);
    };

    return {
        isDetached,
        isMinimized,
        constraintsRef,
        xOpen,
        yOpen,
        xMin,
        yMin,
        setDetached,
        toggleDetach,
        performMinimizeTransform,
        performMaximizeTransform
    };
}
