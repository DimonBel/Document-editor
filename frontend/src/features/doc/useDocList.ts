import { useState, useCallback, useEffect } from 'react';
import { docsApi } from '../../core/api/docsApi';
import { DocInfo } from '../../types';

export function useDocList() {
  const [docs, setDocs] = useState<DocInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchDocs = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const list = await docsApi.list();
      setDocs(list);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to fetch documents');
    } finally {
      setLoading(false);
    }
  }, []);

  const createDoc = useCallback(async (title: string) => {
    setLoading(true);
    try {
      const doc = await docsApi.create(title);
      await fetchDocs();
      return doc;
    } finally {
      setLoading(false);
    }
  }, [fetchDocs]);

  const getDoc = useCallback((id: string) => docsApi.get(id), []);

  useEffect(() => {
    fetchDocs();
  }, [fetchDocs]);

  return { docs, loading, error, createDoc, getDoc, refresh: fetchDocs };
}