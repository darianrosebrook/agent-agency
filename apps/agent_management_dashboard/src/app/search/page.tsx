"use client";

import { useState, useEffect, useRef } from "react";
import { Search, X, FileText, MessageSquare, FolderKanban, Users, Loader2 } from "lucide-react";
import { useRouter } from "next/navigation";
import { search, type SearchResult, type SearchResponse } from "../../lib/api/search";
import { useDebounce } from "../../hooks/useDebounce";
import styles from "./page.module.scss";

export default function SearchPage() {
  const router = useRouter();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [selectedType, setSelectedType] = useState<'all' | 'project' | 'task' | 'chat' | 'file' | 'agent'>('all');
  const [total, setTotal] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const debouncedQuery = useDebounce(query, 300);

  useEffect(() => {
    // Focus input on mount
    inputRef.current?.focus();

    // Handle "/" keyboard shortcut to focus search
    const handleKeyDown = (e: KeyboardEvent) => {
      // Only trigger if not typing in an input/textarea
      if (e.key === '/' && document.activeElement?.tagName !== 'INPUT' && document.activeElement?.tagName !== 'TEXTAREA') {
        e.preventDefault();
        inputRef.current?.focus();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  useEffect(() => {
    if (!debouncedQuery.trim()) {
      setResults([]);
      setTotal(0);
      return;
    }

    const performSearch = async () => {
      setIsLoading(true);
      setError(null);

      try {
        const response: SearchResponse = await search(debouncedQuery, {
          type: selectedType === 'all' ? undefined : selectedType,
          limit: 50,
        });

        setResults(response.results);
        setTotal(response.total);
      } catch (err) {
        console.error("Search error:", err);
        setError(err instanceof Error ? err : new Error("Failed to search"));
        setResults([]);
        setTotal(0);
      } finally {
        setIsLoading(false);
      }
    };

    performSearch();
  }, [debouncedQuery, selectedType]);

  const handleResultClick = (result: SearchResult) => {
    router.push(result.url);
  };

  const getTypeIcon = (type: SearchResult['type']) => {
    switch (type) {
      case 'project':
        return <FolderKanban className={styles.resultIcon} />;
      case 'task':
        return <FileText className={styles.resultIcon} />;
      case 'chat':
        return <MessageSquare className={styles.resultIcon} />;
      case 'file':
        return <FileText className={styles.resultIcon} />;
      case 'agent':
        return <Users className={styles.resultIcon} />;
      default:
        return <FileText className={styles.resultIcon} />;
    }
  };

  const highlightText = (text: string, query: string) => {
    if (!query.trim()) return text;
    
    const parts = text.split(new RegExp(`(${query})`, 'gi'));
    return parts.map((part, i) => 
      part.toLowerCase() === query.toLowerCase() ? (
        <mark key={i} className={styles.highlight}>{part}</mark>
      ) : (
        part
      )
    );
  };

  return (
    <div className={styles.searchPage}>
      <div className={styles.searchContainer}>
        {/* Search Header */}
        <div className={styles.searchHeader}>
          <div className={styles.searchInputContainer}>
            <Search className={styles.searchIcon} />
            <input
              ref={inputRef}
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search everything..."
              className={styles.searchInput}
            />
            {query && (
              <button
                onClick={() => setQuery("")}
                className={styles.clearButton}
                aria-label="Clear search"
              >
                <X className={styles.clearIcon} />
              </button>
            )}
          </div>

          {/* Type Filter */}
          <div className={styles.typeFilter}>
            {(['all', 'project', 'task', 'chat', 'file', 'agent'] as const).map((type) => (
              <button
                key={type}
                onClick={() => setSelectedType(type)}
                className={`${styles.typeButton} ${selectedType === type ? styles.typeButtonActive : ''}`}
              >
                {type.charAt(0).toUpperCase() + type.slice(1)}
              </button>
            ))}
          </div>
        </div>

        {/* Results */}
        <div className={styles.resultsContainer}>
          {isLoading && (
            <div className={styles.loadingState}>
              <Loader2 className={styles.loadingIcon} />
              <p>Searching...</p>
            </div>
          )}

          {error && (
            <div className={styles.errorState}>
              <p className={styles.errorMessage}>Error: {error.message}</p>
            </div>
          )}

          {!isLoading && !error && query && results.length === 0 && (
            <div className={styles.emptyState}>
              <p>No results found for &quot;{query}&quot;</p>
            </div>
          )}

          {!isLoading && !error && query && results.length > 0 && (
            <>
              <div className={styles.resultsHeader}>
                <p className={styles.resultsCount}>
                  {total} result{total !== 1 ? 's' : ''} found
                </p>
              </div>
              <div className={styles.resultsList}>
                {results.map((result) => (
                  <div
                    key={result.id}
                    onClick={() => handleResultClick(result)}
                    className={styles.resultItem}
                  >
                    <div className={styles.resultIconContainer}>
                      {getTypeIcon(result.type)}
                    </div>
                    <div className={styles.resultContent}>
                      <h3 className={styles.resultTitle}>
                        {highlightText(result.title, query)}
                      </h3>
                      {result.description && (
                        <p className={styles.resultDescription}>
                          {highlightText(result.description, query)}
                        </p>
                      )}
                      <div className={styles.resultMetadata}>
                        <span className={styles.resultType}>{result.type}</span>
                        {result.metadata && (
                          <span className={styles.resultMeta}>
                            {Object.entries(result.metadata)
                              .filter(([_, v]) => v !== null && v !== undefined)
                              .slice(0, 2)
                              .map(([k, v]) => `${k}: ${v}`)
                              .join(' • ')}
                          </span>
                        )}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </>
          )}

          {!query && (
            <div className={styles.initialState}>
              <Search className={styles.initialIcon} />
              <h2>Search Everything</h2>
              <p>Search across projects, tasks, chats, files, and agents using vector and knowledge search.</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

