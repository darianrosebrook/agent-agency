/**
 * Global Search Component
 * Provides search across tasks, metrics, and settings
 * 
 * @author @darianrosebrook
 */

"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import { Search, X, Clock, ArrowRight } from "lucide-react";
import { Text } from "@/design-system/primitives";
import { useRouter } from "next/navigation";
import styles from "./GlobalSearch.module.scss";

export interface SearchResult {
  id: string;
  title: string;
  description: string;
  type: "task" | "metric" | "setting" | "page";
  url: string;
  category: string;
  icon?: React.ReactNode;
}

interface GlobalSearchProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function GlobalSearch({ isOpen, onClose }: GlobalSearchProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [recentSearches, setRecentSearches] = useState<string[]>([]);
  
  const inputRef = useRef<HTMLInputElement>(null);
  const router = useRouter();

  // Load recent searches from localStorage
  useEffect(() => {
    const saved = localStorage.getItem("recent-searches");
    if (saved) {
      try {
        setRecentSearches(JSON.parse(saved));
      } catch (error) {
        console.error("Failed to load recent searches:", error);
      }
    }
  }, []);

  // Focus input when opened
  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  // Search function
  const performSearch = useCallback(async (searchQuery: string) => {
    if (!searchQuery.trim()) {
      setResults([]);
      return;
    }

    setIsLoading(true);
    
    try {
      // Mock search results - in real implementation, this would call an API
      const mockResults: SearchResult[] = [
        {
          id: "task-1",
          title: "Process User Data",
          description: "Task to process user data for analytics",
          type: "task",
          url: "/tasks/task-1",
          category: "Tasks",
        },
        {
          id: "metric-1",
          title: "System Health",
          description: "Overall system health metrics",
          type: "metric",
          url: "/metrics",
          category: "Metrics",
        },
        {
          id: "setting-1",
          title: "Notification Settings",
          description: "Configure notification preferences",
          type: "setting",
          url: "/settings",
          category: "Settings",
        },
        {
          id: "page-1",
          title: "Data Quality Dashboard",
          description: "Monitor database health and data integrity",
          type: "page",
          url: "/data-quality",
          category: "Pages",
        },
      ];

      // Filter results based on query
      const filteredResults = mockResults.filter(
        (result) =>
          result.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
          result.description.toLowerCase().includes(searchQuery.toLowerCase())
      );

      setResults(filteredResults);
    } catch (error) {
      console.error("Search failed:", error);
      setResults([]);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Handle search input
  const handleSearch = useCallback((value: string) => {
    setQuery(value);
    setSelectedIndex(0);
    performSearch(value);
  }, [performSearch]);

  // Handle result selection
  const handleResultSelect = useCallback((result: SearchResult) => {
    // Save to recent searches
    const newRecent = [result.title, ...recentSearches.filter(item => item !== result.title)].slice(0, 5);
    setRecentSearches(newRecent);
    localStorage.setItem("recent-searches", JSON.stringify(newRecent));
    
    // Navigate to result
    router.push(result.url);
    onClose();
  }, [recentSearches, router, onClose]);

  // Handle keyboard navigation
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (!isOpen) return;

    switch (e.key) {
      case "Escape":
        onClose();
        break;
      case "ArrowDown":
        e.preventDefault();
        setSelectedIndex(prev => Math.min(prev + 1, results.length - 1));
        break;
      case "ArrowUp":
        e.preventDefault();
        setSelectedIndex(prev => Math.max(prev - 1, -1));
        break;
      case "Enter":
        e.preventDefault();
        if (selectedIndex >= 0 && results[selectedIndex]) {
          handleResultSelect(results[selectedIndex]);
        }
        break;
    }
  }, [isOpen, results, selectedIndex, handleResultSelect, onClose]);

  // Get result icon
  const getResultIcon = (type: SearchResult["type"]) => {
    switch (type) {
      case "task":
        return "";
      case "metric":
        return "";
      case "setting":
        return "⚙️";
      case "page":
        return "";
      default:
        return "";
    }
  };

  if (!isOpen) return null;

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div className={styles.searchModal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.searchInput}>
          <Search className={styles.searchIcon} />
          <input
            ref={inputRef}
            type="text"
            placeholder="Search tasks, metrics, settings..."
            value={query}
            onChange={(e) => handleSearch(e.target.value)}
            onKeyDown={handleKeyDown}
            className={styles.input}
            autoComplete="off"
          />
          {query && (
            <button
              onClick={() => setQuery("")}
              className={styles.clearButton}
              aria-label="Clear search"
            >
              <X size={16} />
            </button>
          )}
        </div>

        <div className={styles.results}>
          {isLoading ? (
            <div className={styles.loading}>
              <div className={styles.spinner}></div>
              <Text variant="paragraph-small" color="secondary">
                Searching...
              </Text>
            </div>
          ) : query ? (
            results.length > 0 ? (
              <div className={styles.resultsList}>
                {results.map((result, index) => (
                  <button
                    key={result.id}
                    onClick={() => handleResultSelect(result)}
                    className={`${styles.resultItem} ${
                      index === selectedIndex ? styles.selected : ""
                    }`}
                  >
                    <div className={styles.resultIcon}>
                      {getResultIcon(result.type)}
                    </div>
                    <div className={styles.resultContent}>
                      <Text variant="paragraph-medium" weight="medium">
                        {result.title}
                      </Text>
                      <Text variant="paragraph-small" color="secondary">
                        {result.description}
                      </Text>
                      <Text variant="caption" color="muted">
                        {result.category}
                      </Text>
                    </div>
                    <ArrowRight className={styles.resultArrow} />
                  </button>
                ))}
              </div>
            ) : (
              <div className={styles.noResults}>
                <Text variant="paragraph-medium" color="secondary">
                  No results found for "{query}"
                </Text>
              </div>
            )
          ) : (
            <div className={styles.recentSearches}>
              <Text variant="paragraph-small" weight="medium" color="secondary">
                Recent searches
              </Text>
              {recentSearches.length > 0 ? (
                <div className={styles.recentList}>
                  {recentSearches.map((search, index) => (
                    <button
                      key={index}
                      onClick={() => handleSearch(search)}
                      className={styles.recentItem}
                    >
                      <Clock className={styles.recentIcon} />
                      <Text variant="paragraph-small">
                        {search}
                      </Text>
                    </button>
                  ))}
                </div>
              ) : (
                <Text variant="paragraph-small" color="muted">
                  No recent searches
                </Text>
              )}
            </div>
          )}
        </div>

        <div className={styles.footer}>
          <Text variant="caption" color="muted">
            Press <kbd>↑↓</kbd> to navigate, <kbd>Enter</kbd> to select, <kbd>Esc</kbd> to close
          </Text>
        </div>
      </div>
    </div>
  );
}
