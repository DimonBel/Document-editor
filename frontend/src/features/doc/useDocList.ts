import { useState, useCallback } from 'react';
import { docsApi } from '../../core/api/docsApi';
import { DocInfo } from '../../types';

export function useDocList() {
  const [docs, setDocs] = useState<DocInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchDocs = useCallback(async () => {
    try {
      setDocs(await docsApi.list());
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to fetch documents');
    } finally {
      setLoading(false);
    }
  }, []);

  const createDoc = useCallback(async (title: string) => {
    const doc = await docsApi.create(title);
    await fetchDocs();
    return doc;
  }, [fetchDocs]);

  const getDoc = useCallback((id: string) => docsApi.get(id), []);

  return { docs, loading, error, createDoc, getDoc, refresh: fetchDocs };
}