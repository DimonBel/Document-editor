import { useState, useEffect } from 'react';

/**
 * Returns the current viewport dimensions, updated on resize.
 * Used to size the Konva Stage responsively.
 */
export function useWindowSize() {
  const [size, setSize] = useState({
    width:  window.innerWidth,
    height: window.innerHeight,
  });

  useEffect(() => {
    const handler = () =>
      setSize({ width: window.innerWidth, height: window.innerHeight });

    window.addEventListener('resize', handler);
    return () => window.removeEventListener('resize', handler);
  }, []);

  return size;
}
