import { useCallback, useState } from 'react';

/** JSON API responses that have not changed should retain their React identity. */
export function useStableData<T>(initial: T) {
  const [value, setValue] = useState(initial);
  const update = useCallback((next: T) => {
    setValue(current => JSON.stringify(current) === JSON.stringify(next) ? current : next);
  }, []);
  return [value, update] as const;
}
