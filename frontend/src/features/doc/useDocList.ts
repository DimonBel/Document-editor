import { useState, useCallback, useEffect, useRef } from 'react';
import { docsApi } from '../../core/api/docsApi';
import { DocInfo } from '../../types';

const POLL_INTERVAL = 8000;

export function useDocList() {
  const [docs, setDocs] = useState<DocInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const mountedRef = useRef(true);

  const fetchDocs = useCallback(async () => {
    if (!mountedRef.current) return;
    setLoading(true);
    setError(null);
    try {
      const list = await docsApi.list();
      if (mountedRef.current) setDocs(list);
    } catch (e) {
      if (mountedRef.current) {
        setError(e instanceof Error ? e.message : 'Failed to fetch documents');
      }
    } finally {
      if (mountedRef.current) setLoading(false);
    }
  }, []);

  const createDoc = useCallback(async (title: string) => {
    const doc = await docsApi.create(title);
    // Refresh in the background; do not gate the caller on it.
    void fetchDocs();
    return doc;
  }, [fetchDocs]);

  const getDoc = useCallback((id: string) => docsApi.get(id), []);

  useEffect(() => {
    mountedRef.current = true;
    void fetchDocs();
    const timer = window.setInterval(() => { void fetchDocs(); }, POLL_INTERVAL);
    return () => {
      mountedRef.current = false;
      window.clearInterval(timer);
    };
  }, [fetchDocs]);

  return { docs, loading, error, createDoc, getDoc, refresh: fetchDocs };
}