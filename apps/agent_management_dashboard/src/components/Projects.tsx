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
import { NewProjectModal } from "./NewProjectModal";
import { ProjectView } from "./ProjectView";
import { Input } from "./ui/input";
import { Button } from "./ui/button";
import { useProjectContext } from "./ProjectContext";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "./ui/table";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";

type SortField = "name" | "createdAt" | "lastAccessed";
type SortOrder = "asc" | "desc";

export function Projects() {
  const {
    projects,
    getCurrentProject,
    createProject,
    selectProject,
    clearCurrentProject,
  } = useProjectContext();
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
    let aValue: string | Date = a[sortField];
    let bValue: string | Date = b[sortField];

    if (sortField === "createdAt" || sortField === "lastAccessed") {
      aValue = aValue.getTime();
      bValue = bValue.getTime();
    } else if (sortField === "name") {
      aValue = aValue.toLowerCase();
      bValue = bValue.toLowerCase();
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
      <div className="p-8">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center gap-2 text-gray-300 mb-4">
            <FolderPlus className="w-4 h-4" />
            <span className="text-sm">Projects</span>
          </div>
          <h1 className="text-3xl text-white">Projects</h1>
        </div>

        {/* Empty State */}
        <div className="flex items-center justify-center min-h-[500px]">
          <div className="text-center max-w-3xl w-full">
            {/* Icon as Button */}
            <div className="mb-6 flex justify-center">
              <button
                onClick={() => setIsNewProjectModalOpen(true)}
                className="relative group cursor-pointer"
              >
                <div className="w-32 h-32 bg-[#1a1a1a] border-2 border-gray-800 rounded-3xl flex items-center justify-center group-hover:border-blue-500/50 group-hover:bg-[#1f1f1f] transition-all">
                  <FolderPlus className="w-16 h-16 text-gray-700 group-hover:text-blue-500 transition-colors" />
                </div>
              </button>
            </div>

            {/* Text */}
            <h2 className="text-2xl text-white mb-3">No projects yet</h2>
            <p className="text-gray-400 mb-8">
              Create your first project to get started. Projects help you
              organize your work and collaborate with your team.
            </p>

            {/* Create Project Button */}
            <Button
              onClick={() => setIsNewProjectModalOpen(true)}
              className="bg-blue-600 text-white hover:bg-blue-700"
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
    <div className="p-8">
      {/* Header */}
      <div className="mb-8">
        <div className="flex items-center justify-between mb-4">
          <div>
            <div className="flex items-center gap-2 text-gray-300 mb-4">
              <FolderPlus className="w-4 h-4" />
              <span className="text-sm">Projects</span>
            </div>
            <h1 className="text-3xl text-white">Projects</h1>
          </div>
          <Button
            onClick={() => setIsNewProjectModalOpen(true)}
            className="bg-blue-600 text-white hover:bg-blue-700"
          >
            <Plus className="w-4 h-4 mr-2" />
            New Project
          </Button>
        </div>

        {/* Search and Filter Bar */}
        <div className="flex items-center gap-3">
          <div className="relative flex-1 max-w-md">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
            <Input
              placeholder="Search projects..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-10 bg-[#1a1a1a] border-gray-800 text-gray-100 placeholder:text-gray-600"
            />
          </div>
          <Button
            variant="outline"
            className="bg-[#1a1a1a] border-gray-800 text-gray-300 hover:bg-gray-800 hover:text-gray-100"
          >
            <Filter className="w-4 h-4 mr-2" />
            Filter
          </Button>
        </div>
      </div>

      {/* Recent Projects */}
      <div className="mb-8">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl text-white flex items-center gap-2">
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

        {recentProjects.length === 0 ? (
          <div className="text-center py-12 text-gray-500">
            No projects found
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {recentProjects.map((project) => (
              <button
                key={project.id}
                onClick={() => handleProjectClick(project.id)}
                className="bg-[#1a1a1a] border border-gray-800 rounded-lg p-6 text-left hover:bg-[#1f1f1f] hover:border-gray-700 transition-all group"
              >
                <div className="flex items-start gap-4">
                  <div className="w-12 h-12 bg-[#0f0f0f] border border-gray-800 rounded-lg flex items-center justify-center group-hover:border-blue-500/50 transition-colors">
                    <FolderPlus className="w-6 h-6 text-gray-600 group-hover:text-blue-500 transition-colors" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <h3 className="text-white mb-1 truncate">{project.name}</h3>
                    {project.summary && (
                      <p className="text-sm text-gray-500 mb-1 truncate">
                        {project.summary}
                      </p>
                    )}
                    <p className="text-xs text-gray-600">
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
      <div className="mb-8">
        <h2 className="text-xl text-white mb-4">All Projects</h2>

        <div className="bg-[#1a1a1a] border border-gray-800 rounded-lg overflow-hidden">
          <Table>
            <TableHeader>
              <TableRow className="border-gray-800 hover:bg-transparent">
                <TableHead
                  className="text-gray-400 cursor-pointer select-none hover:text-gray-200 transition-colors"
                  onClick={() => handleSort("name")}
                >
                  <div className="flex items-center gap-2">
                    Name
                    {getSortIcon("name")}
                  </div>
                </TableHead>
                <TableHead className="text-gray-400">Summary</TableHead>
                <TableHead
                  className="text-gray-400 cursor-pointer select-none hover:text-gray-200 transition-colors"
                  onClick={() => handleSort("createdAt")}
                >
                  <div className="flex items-center gap-2">
                    Created
                    {getSortIcon("createdAt")}
                  </div>
                </TableHead>
                <TableHead
                  className="text-gray-400 cursor-pointer select-none hover:text-gray-200 transition-colors"
                  onClick={() => handleSort("lastAccessed")}
                >
                  <div className="flex items-center gap-2">
                    Last Accessed
                    {getSortIcon("lastAccessed")}
                  </div>
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {paginatedProjects.length === 0 ? (
                <TableRow className="hover:bg-transparent border-gray-800">
                  <TableCell
                    colSpan={4}
                    className="text-center py-12 text-gray-500"
                  >
                    No projects found
                  </TableCell>
                </TableRow>
              ) : (
                paginatedProjects.map((project) => (
                  <TableRow
                    key={project.id}
                    className="border-gray-800 hover:bg-[#1f1f1f] cursor-pointer transition-colors"
                    onClick={() => handleProjectClick(project.id)}
                  >
                    <TableCell className="text-white">
                      <div className="flex items-center gap-3">
                        <div className="w-8 h-8 bg-[#0f0f0f] border border-gray-800 rounded flex items-center justify-center">
                          <FolderPlus className="w-4 h-4 text-gray-600" />
                        </div>
                        {project.name}
                      </div>
                    </TableCell>
                    <TableCell className="text-gray-400 max-w-md truncate">
                      {project.summary ?? "—"}
                    </TableCell>
                    <TableCell className="text-gray-400">
                      {formatDate(project.createdAt)}
                    </TableCell>
                    <TableCell className="text-gray-400">
                      {formatDate(project.lastAccessed)}
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>

          {/* Pagination Controls */}
          {sortedProjects.length > 0 && (
            <div className="flex items-center justify-between px-6 py-4 border-t border-gray-800">
              <div className="flex items-center gap-2 text-sm text-gray-400">
                <span>Show</span>
                <Select
                  value={pageSize.toString()}
                  onValueChange={(value) => {
                    setPageSize(parseInt(value));
                    setCurrentPage(1);
                  }}
                >
                  <SelectTrigger className="w-20 h-8 bg-[#0f0f0f] border-gray-800 text-gray-300">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent className="bg-[#1a1a1a] border-gray-800">
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

              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setCurrentPage(currentPage - 1)}
                  disabled={currentPage === 1}
                  className="bg-[#0f0f0f] border-gray-800 text-gray-300 hover:bg-gray-800 hover:text-gray-100 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Previous
                </Button>
                <span className="text-sm text-gray-400">
                  Page {currentPage} of {totalPages}
                </span>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setCurrentPage(currentPage + 1)}
                  disabled={currentPage === totalPages}
                  className="bg-[#0f0f0f] border-gray-800 text-gray-300 hover:bg-gray-800 hover:text-gray-100 disabled:opacity-50 disabled:cursor-not-allowed"
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
