"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import {
  FolderPlus,
  Plus,
  Search,
  Filter,
  Clock,
  ChevronDown,
  ChevronUp,
  ChevronsUpDown,
} from "lucide-react";
import { NewProjectModal } from "./NewProjectModal";
import { Input } from "./primitives/input";
import { Button } from "./primitives/button";
import { useProjectStore } from "../lib/stores";
import { ErrorDisplay } from "./ErrorDisplay";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "./primitives/table";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./primitives/select";
import styles from "./Projects.module.scss";

type SortField = "name" | "createdAt" | "lastAccessed";
type SortOrder = "asc" | "desc";

export function Projects() {
  const router = useRouter();
  const {
    projects,
    getCurrentProject,
    createProject,
    selectProject,
    isLoading,
    error,
  } = useProjectStore();
  const [isNewProjectModalOpen, setIsNewProjectModalOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [showAllRecent, setShowAllRecent] = useState(false);

  // Table state
  const [sortField, setSortField] = useState<SortField>("lastAccessed");
  const [sortOrder, setSortOrder] = useState<SortOrder>("desc");
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState(10);

  const currentProject = getCurrentProject();

  // Show loading state
  if (isLoading && projects.length === 0) {
    return (
      <div className={styles.loadingContainer}>
        <div className={styles.loadingContent}>
          <div className={styles.loadingSpinner}>
            <div className={styles.spinner}></div>
            <p className={styles.loadingText}>Loading projects...</p>
          </div>
        </div>
      </div>
    );
  }

  // Show error state
  if (error && projects.length === 0) {
    return (
      <div className={styles.projectsContainer}>
        <ErrorDisplay
          error={error}
          onRetry={async () => {
            // Retry fetching projects
            try {
              await useProjectStore.getState().fetchProjects();
            } catch {
              // Error already handled in store
            }
          }}
        />
      </div>
    );
  }

  const handleCreateProject = (data: {
    name: string;
    summary?: string;
    description?: string;
    milestones?: string[];
  }) => {
    createProject(data);
  };

  const handleProjectClick = (projectId: string) => {
    if (!projectId || typeof projectId !== "string") {
      console.error("Invalid projectId:", projectId);
      return;
    }
    selectProject(projectId);
    router.push(`/projects/${encodeURIComponent(projectId)}`);
  };

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortOrder(sortOrder === "asc" ? "desc" : "asc");
    } else {
      setSortField(field);
      setSortOrder("desc");
    }
    setCurrentPage(1);
  };

  const getSortIcon = (field: SortField) => {
    if (sortField !== field) {
      return <ChevronsUpDown className={styles.tableHeaderIcon} />;
    }
    return sortOrder === "asc" ? (
      <ChevronUp className={styles.tableHeaderIcon} />
    ) : (
      <ChevronDown className={styles.tableHeaderIcon} />
    );
  };

  // Filter projects based on search
  const filteredProjects = projects.filter((p) =>
    p.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  // Sort projects by last accessed for recent section
  const recentProjects = [...filteredProjects]
    .sort((a, b) => b.lastAccessed.getTime() - a.lastAccessed.getTime())
    .slice(0, showAllRecent ? filteredProjects.length : 6);

  // Sort and paginate projects for table
  const sortedProjects = [...filteredProjects].sort((a, b) => {
    let aValue: string | number;
    let bValue: string | number;
    const aFieldValue = a[sortField];
    const bFieldValue = b[sortField];

    if (sortField === "createdAt" || sortField === "lastAccessed") {
      const aDate =
        aFieldValue instanceof Date
          ? aFieldValue
          : new Date(aFieldValue as string);
      const bDate =
        bFieldValue instanceof Date
          ? bFieldValue
          : new Date(bFieldValue as string);
      aValue = aDate.getTime();
      bValue = bDate.getTime();
    } else if (sortField === "name") {
      aValue = String(aFieldValue).toLowerCase();
      bValue = String(bFieldValue).toLowerCase();
    } else {
      aValue = String(aFieldValue);
      bValue = String(bFieldValue);
    }

    if (sortOrder === "asc") {
      return aValue > bValue ? 1 : -1;
    } else {
      return aValue < bValue ? 1 : -1;
    }
  });

  const totalPages = Math.ceil(sortedProjects.length / pageSize);
  const paginatedProjects = sortedProjects.slice(
    (currentPage - 1) * pageSize,
    currentPage * pageSize
  );

  const formatDate = (date: Date) => {
    const now = new Date();
    const diffInMs = now.getTime() - date.getTime();
    const diffInHours = diffInMs / (1000 * 60 * 60);
    const diffInDays = diffInMs / (1000 * 60 * 60 * 24);

    if (diffInHours < 1) {
      return "Just now";
    } else if (diffInHours < 24) {
      return `${Math.floor(diffInHours)} hours ago`;
    } else if (diffInDays < 7) {
      return `${Math.floor(diffInDays)} days ago`;
    } else {
      return date.toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
        year: date.getFullYear() !== now.getFullYear() ? "numeric" : undefined,
      });
    }
  };

  // Empty State View
  if (!currentProject && projects.length === 0) {
    return (
      <div className={styles.projectsContainer}>
        {/* Header */}
        <div className={styles.header}>
          <div className={styles.headerTop}>
            <FolderPlus className={styles.icon} />
            <span className="text-sm">Projects</span>
          </div>
          <h1 className={styles.headerTitle}>Projects</h1>
        </div>

        {/* Empty State */}
        <div className={styles.emptyStateContainer}>
          <div className={styles.emptyStateContent}>
            {/* Icon as Button */}
            <div className={styles.emptyStateIcon}>
              <button
                onClick={() => setIsNewProjectModalOpen(true)}
                className={styles.emptyStateIconButton}
              >
                <div className={styles.emptyStateIconBox}>
                  <FolderPlus className={styles.emptyStateIconLarge} />
                </div>
              </button>
            </div>

            {/* Text */}
            <h2 className={styles.emptyStateTitle}>No projects yet</h2>
            <p className={styles.emptyStateDescription}>
              Create your first project to get started. Projects help you
              organize your work and collaborate with your team.
            </p>

            {/* Create Project Button */}
            <Button
              onClick={() => setIsNewProjectModalOpen(true)}
              className={styles.newProjectButton}
            >
              <Plus className={styles.iconWithMargin} />
              Create Project
            </Button>
          </div>
        </div>

        <NewProjectModal
          open={isNewProjectModalOpen}
          onOpenChange={setIsNewProjectModalOpen}
          onCreateProject={handleCreateProject}
        />
      </div>
    );
  }

  // Projects List View
  return (
    <div className={styles.projectsContainer}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerWithButton}>
          <div className={styles.headerLeft}>
            <div className={styles.headerTop}>
              <FolderPlus className={styles.icon} />
              <span className="text-sm">Projects</span>
            </div>
            <h1 className={styles.headerTitle}>Projects</h1>
          </div>
          <Button
            onClick={() => setIsNewProjectModalOpen(true)}
            className={styles.newProjectButton}
          >
            <Plus className={styles.iconWithMargin} />
            New Project
          </Button>
        </div>

        {/* Search and Filter Bar */}
        <div className={styles.searchFilterBar}>
          <div className={styles.searchContainer}>
            <Search className={styles.searchIcon} />
            <Input
              placeholder="Search projects..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className={styles.searchInput}
            />
          </div>
          <Button variant="outline" className={styles.filterButton}>
            <Filter className={styles.iconWithMargin} />
            Filter
          </Button>
        </div>
      </div>

      {/* Recent Projects */}
      <div className={styles.recentProjectsSection}>
        <div className={styles.recentProjectsHeader}>
          <h2 className={styles.recentProjectsTitle}>
            <Clock className={styles.recentProjectsIcon} />
            Recent Projects
          </h2>
          {filteredProjects.length > 6 && (
            <Button
              variant="ghost"
              onClick={() => setShowAllRecent(!showAllRecent)}
              className={styles.recentProjectsSeeMore}
            >
              {showAllRecent ? "Show Less" : "See More"}
            </Button>
          )}
        </div>

        {recentProjects.length === 0 ? (
          <div className={styles.recentProjectsEmpty}>No projects found</div>
        ) : (
          <div className={styles.recentProjectsGrid}>
            {recentProjects.map((project) => (
              <button
                key={project.id}
                onClick={() => handleProjectClick(project.id)}
                className={styles.projectCard}
              >
                <div className={styles.projectCardContent}>
                  <div className={styles.projectCardIcon}>
                    <FolderPlus className={styles.projectCardIconInner} />
                  </div>
                  <div className={styles.projectCardDetails}>
                    <h3 className={styles.projectCardName}>{project.name}</h3>
                    {project.summary && (
                      <p className={styles.projectCardSummary}>
                        {project.summary}
                      </p>
                    )}
                    <p className={styles.projectCardDate}>
                      {formatDate(project.lastAccessed)}
                    </p>
                  </div>
                </div>
              </button>
            ))}
          </div>
        )}
      </div>

      {/* All Projects Table */}
      <div className={styles.allProjectsSection}>
        <h2 className={styles.allProjectsTitle}>All Projects</h2>

        <div className={styles.projectsTable}>
          <Table>
            <TableHeader>
              <TableRow className={styles.tableHeaderRow}>
                <TableHead
                  className={styles.tableHeaderCell}
                  onClick={() => handleSort("name")}
                >
                  <div className={styles.tableHeaderCellContent}>
                    Name
                    {getSortIcon("name")}
                  </div>
                </TableHead>
                <TableHead className={styles.tableHeaderCell}>Summary</TableHead>
                <TableHead
                  className={styles.tableHeaderCell}
                  onClick={() => handleSort("createdAt")}
                >
                  <div className={styles.tableHeaderCellContent}>
                    Created
                    {getSortIcon("createdAt")}
                  </div>
                </TableHead>
                <TableHead
                  className={styles.tableHeaderCell}
                  onClick={() => handleSort("lastAccessed")}
                >
                  <div className={styles.tableHeaderCellContent}>
                    Last Accessed
                    {getSortIcon("lastAccessed")}
                  </div>
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {paginatedProjects.length === 0 ? (
                <TableRow className={styles.tableRow}>
                  <TableCell colSpan={4} className={styles.tableEmptyCell}>
                    No projects found
                  </TableCell>
                </TableRow>
              ) : (
                paginatedProjects.map((project) => (
                  <TableRow
                    key={project.id}
                    className={styles.tableRow}
                    onClick={() => handleProjectClick(project.id)}
                  >
                    <TableCell className={styles.tableCell}>
                      <div className={styles.tableCellIcon}>
                        <div className={styles.tableCellIconBox}>
                          <FolderPlus className={styles.tableCellIconInner} />
                        </div>
                        {project.name}
                      </div>
                    </TableCell>
                    <TableCell className={styles.tableCellGray}>
                      {project.summary ?? "—"}
                    </TableCell>
                    <TableCell className={styles.tableCellGray}>
                      {formatDate(project.createdAt)}
                    </TableCell>
                    <TableCell className={styles.tableCellGray}>
                      {formatDate(project.lastAccessed)}
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>

          {/* Pagination Controls */}
          {sortedProjects.length > 0 && (
            <div className={styles.pagination}>
              <div className={styles.paginationLeft}>
                <span>Show</span>
                <Select
                  value={pageSize.toString()}
                  onValueChange={(value) => {
                    setPageSize(parseInt(value));
                    setCurrentPage(1);
                  }}
                >
                  <SelectTrigger className={styles.paginationSelect}>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent className={styles.paginationSelectContent}>
                    <SelectItem value="5">5</SelectItem>
                    <SelectItem value="10">10</SelectItem>
                    <SelectItem value="20">20</SelectItem>
                    <SelectItem value="50">50</SelectItem>
                  </SelectContent>
                </Select>
                <span>
                  of {sortedProjects.length}{" "}
                  {sortedProjects.length === 1 ? "project" : "projects"}
                </span>
              </div>

              <div className={styles.paginationRight}>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setCurrentPage(currentPage - 1)}
                  disabled={currentPage === 1}
                  className={styles.paginationButton}
                >
                  Previous
                </Button>
                <span className={styles.paginationInfo}>
                  Page {currentPage} of {totalPages}
                </span>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setCurrentPage(currentPage + 1)}
                  disabled={currentPage === totalPages}
                  className={styles.paginationButton}
                >
                  Next
                </Button>
              </div>
            </div>
          )}
        </div>
      </div>

      <NewProjectModal
        open={isNewProjectModalOpen}
        onOpenChange={setIsNewProjectModalOpen}
        onCreateProject={handleCreateProject}
      />
    </div>
  );
}
