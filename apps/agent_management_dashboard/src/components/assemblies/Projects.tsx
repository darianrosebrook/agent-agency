"use client";

import { useState } from "react";
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
import { NewProjectModal } from "../composers/ProjectModal";
import { ProjectView } from "./ProjectView";
import { Input } from "../ui/input";
import { Button } from "../ui/button";
import { useProjectStore } from "../../lib/stores";
import { ProjectListSkeleton } from "../compounds";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../ui/table";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import { cn } from "../ui/utils";
import styles from "./Projects.module.scss";

type SortField = "name" | "createdAt" | "lastAccessed";
type SortOrder = "asc" | "desc";

export function Projects() {
  const {
    projects,
    getCurrentProject,
    createProject,
    selectProject,
    clearCurrentProject,
    isLoading,
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

  const handleCreateProject = (data: {
    name: string;
    summary?: string;
    description?: string;
    milestones?: string[];
  }) => {
    createProject(data);
  };

  const handleProjectClick = (projectId: string) => {
    selectProject(projectId);
  };

  const handleBackToProjects = () => {
    clearCurrentProject();
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
      return <ChevronsUpDown className="w-4 h-4" />;
    }
    return sortOrder === "asc" ? (
      <ChevronUp className="w-4 h-4" />
    ) : (
      <ChevronDown className="w-4 h-4" />
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
      const aDate = aFieldValue instanceof Date ? aFieldValue : new Date(aFieldValue as string);
      const bDate = bFieldValue instanceof Date ? bFieldValue : new Date(bFieldValue as string);
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
            <FolderPlus className="w-4 h-4" />
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
                  <FolderPlus className="w-16 h-16 text-gray-700 group-hover:text-blue-500 transition-colors" />
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
              <Plus className="w-4 h-4 mr-2" />
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

  // Project View (when a project is selected)
  if (currentProject) {
    return (
      <ProjectView
        projectName={currentProject.name}
        onBackToProjects={handleBackToProjects}
      />
    );
  }

  // Projects List View (when there are projects but none selected)
  return (
    <div className={styles.projectsContainer}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerWithButton}>
          <div className={styles.headerLeft}>
            <div className={styles.headerTop}>
              <FolderPlus className="w-4 h-4" />
              <span className="text-sm">Projects</span>
            </div>
            <h1 className={styles.headerTitle}>Projects</h1>
          </div>
          <Button
            onClick={() => setIsNewProjectModalOpen(true)}
            className={styles.newProjectButton}
          >
            <Plus className="w-4 h-4 mr-2" />
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
              className="pl-10 bg-[#1a1a1a] border-gray-800 text-gray-100 placeholder:text-gray-600"
            />
          </div>
          <Button
            variant="outline"
            className={styles.filterButton}
          >
            <Filter className="w-4 h-4 mr-2" />
            Filter
          </Button>
        </div>
      </div>

      {/* Recent Projects */}
      <div className={styles.recentProjectsSection}>
        <div className={styles.recentProjectsHeader}>
          <h2 className={styles.recentProjectsTitle}>
            <Clock className="w-5 h-5 text-gray-400" />
            Recent Projects
          </h2>
          {filteredProjects.length > 6 && (
            <Button
              variant="ghost"
              onClick={() => setShowAllRecent(!showAllRecent)}
              className="text-blue-500 hover:text-blue-400 hover:bg-transparent"
            >
              {showAllRecent ? "Show Less" : "See More"}
            </Button>
          )}
        </div>

        {isLoading ? (
          <ProjectListSkeleton count={6} />
        ) : recentProjects.length === 0 ? (
          <div className={styles.recentProjectsEmpty}>
            No projects found
          </div>
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
                    <FolderPlus className="w-6 h-6 text-gray-600 group-hover:text-blue-500 transition-colors" />
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
              <TableRow className={cn(styles.tableHeaderRow, "border-gray-800")}>
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
              {isLoading ? (
                <TableRow className={cn(styles.tableRowLoading, "border-gray-800")}>
                  <TableCell colSpan={4} className="py-8">
                    <div className="flex items-center justify-center">
                      <ProjectListSkeleton count={pageSize} />
                    </div>
                  </TableCell>
                </TableRow>
              ) : paginatedProjects.length === 0 ? (
                <TableRow className={cn(styles.tableRowLoading, "border-gray-800")}>
                  <TableCell
                    colSpan={4}
                    className={styles.tableEmptyCell}
                  >
                    No projects found
                  </TableCell>
                </TableRow>
              ) : (
                paginatedProjects.map((project) => (
                  <TableRow
                    key={project.id}
                    className={cn(styles.tableRow, "border-gray-800")}
                    onClick={() => handleProjectClick(project.id)}
                  >
                    <TableCell className={styles.tableCell}>
                      <div className={styles.tableCellIcon}>
                        <div className={styles.tableCellIconBox}>
                          <FolderPlus className="w-4 h-4 text-gray-600" />
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
