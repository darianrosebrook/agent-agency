"use client";

import React, { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogDescription,
} from "./primitives/dialog";
import { Upload, FileText } from "lucide-react";
import { Button } from "./primitives/button";
import { cn } from "./primitives/utils";
import styles from "./FileDropzoneModal.module.scss";

interface FileDropzoneModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onFilesAdded: (files: string[]) => void;
}

export function FileDropzoneModal({
  open,
  onOpenChange,
  onFilesAdded,
}: FileDropzoneModalProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [isUploading, setIsUploading] = useState(false);

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = () => {
    setIsDragging(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    // Simulate file upload
    setIsUploading(true);

    // TODO: Replace mock file upload with real file upload to v3 file operations service with the following requirements:
    // 1. File upload: Upload files to v3 file operations service
    //    - Data source: POST /api/projects/:projectId/files/upload endpoint in `iterations/v3/data-infrastructure/src/file_operations` service
    //    - Handle file upload with progress tracking
    //    - Support multiple file types and sizes
    // 2. File metadata persistence: Save file metadata to database
    //    - Data source: POST /api/projects/:projectId/files endpoint to persist file metadata
    //    - Database table: PostgreSQL `project_files` or similar table
    //    - Store file names, paths, sizes, and upload timestamps
    // 3. File structure update: Update project file tree after upload
    //    - Refresh file tree from GET /api/projects/:projectId/files endpoint
    //    - Update workspace file tree component with new files
    // 4. Error handling: Handle upload errors and display user-friendly messages
    //    - Validate file types and sizes before upload
    //    - Display error messages for failed uploads
    //    - Provide retry functionality for failed uploads
    // Simulate processing delay
    setTimeout(() => {
      const mockFiles = ["design-system.sketch", "components.tsx", "README.md"];
      onFilesAdded(mockFiles);
      setIsUploading(false);
      onOpenChange(false);
    }, 2000);
  };

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      setIsUploading(true);

      setTimeout(() => {
        const fileNames = Array.from(e.target.files!).map((f) => f.name);
        onFilesAdded(fileNames);
        setIsUploading(false);
        onOpenChange(false);
      }, 2000);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className={styles.dialogContent}>
        <DialogTitle className="sr-only">
          {isUploading ? "Importing files" : "Upload files"}
        </DialogTitle>
        <DialogDescription className="sr-only">
          {isUploading
            ? "Please wait while we import your files"
            : "Drag and drop files or browse to upload"}
        </DialogDescription>
        {!isUploading ? (
          <div
            className={cn(
              styles.dropzone,
              isDragging ? styles.dropzoneDragging : styles.dropzoneIdle
            )}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
          >
            <div className={styles.iconContainer}>
              <div className={styles.iconInner}>
                <FileText className={styles.iconSvg} />
              </div>
            </div>

            <h3 className={styles.title}>Release file</h3>
            <p className={styles.description}>
              Release your files here to upload
            </p>

            <label className={styles.browseButtonLabel}>
              <input
                type="file"
                multiple
                onChange={handleFileSelect}
                className="hidden"
                {...({
                  webkitdirectory: "",
                  directory: "",
                } as React.InputHTMLAttributes<HTMLInputElement>)}
              />
              <div className={styles.browseButton}>Browse Files</div>
            </label>
          </div>
        ) : (
          <div className={styles.uploadingContainer}>
            <div className={styles.uploadingIconContainer}>
              <div className={styles.uploadingIconWrapper}>
                <div className={styles.uploadingIcon}>
                  <div className={styles.uploadingIconInner}>
                    <Upload className={styles.iconSvgWhite} />
                  </div>
                </div>
              </div>
              {/* Animated dots */}
              <div className={styles.spinnerContainer}>
                {[...Array(8)].map((_, i) => (
                  <div
                    key={i}
                    className={styles.spinnerDot}
                    style={{
                      top: "50%",
                      left: "50%",
                      transform: `rotate(${
                        i * 45
                      }deg) translate(40px) translate(-50%, -50%)`,
                    }}
                  />
                ))}
              </div>
            </div>

            <h3 className={styles.uploadingTitle}>Importing data</h3>
            <p className={styles.uploadingDescription}>
              Please wait a few seconds while we&apos;re
              <br />
              importing your data to the project
            </p>

            <Button
              onClick={() => {
                setIsUploading(false);
                onOpenChange(false);
              }}
              className={styles.cancelButton}
            >
              Cancel
            </Button>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
