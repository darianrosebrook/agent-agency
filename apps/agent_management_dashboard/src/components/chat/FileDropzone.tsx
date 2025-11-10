"use client";

import React, { useState, type DragEvent, type ChangeEvent } from "react";
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogDescription,
} from "../ui/dialog";
import { Upload, FileText } from "lucide-react";
import { Button } from "../ui/button";

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

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = () => {
    setIsDragging(false);
  };

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    // Simulate file upload
    setIsUploading(true);

    // Simulate processing delay
    setTimeout(() => {
      const mockFiles = ["design-system.sketch", "components.tsx", "README.md"];
      onFilesAdded(mockFiles);
      setIsUploading(false);
      onOpenChange(false);
    }, 2000);
  };

  const handleFileSelect = (e: ChangeEvent<HTMLInputElement>) => {
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
      <DialogContent className="bg-[#1a1a1a] border-gray-800 max-w-md">
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
            className={`flex flex-col items-center justify-center py-12 px-6 rounded-lg border-2 border-dashed transition-colors ${
              isDragging
                ? "border-blue-500 bg-blue-500/5"
                : "border-gray-700 bg-[#0f0f0f]"
            }`}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
          >
            <div className="w-20 h-20 bg-gray-800 rounded-full flex items-center justify-center mb-6 relative">
              <div className="w-16 h-16 bg-gray-700 rounded-full flex items-center justify-center">
                <FileText className="w-8 h-8 text-gray-300" />
              </div>
            </div>

            <h3 className="text-white mb-2">Release file</h3>
            <p className="text-gray-400 text-sm text-center mb-6">
              Release your files here to upload
            </p>

            <label className="cursor-pointer">
              <input
                type="file"
                multiple
                onChange={handleFileSelect}
                className="hidden"
                {...({ webkitdirectory: "", directory: "" } as React.InputHTMLAttributes<HTMLInputElement>)}
              />
              <div className="bg-gray-800 hover:bg-gray-700 text-white px-6 py-2 rounded-lg text-sm">
                Browse Files
              </div>
            </label>
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center py-12 px-6">
            <div className="relative w-24 h-24 mb-6">
              <div className="absolute inset-0 flex items-center justify-center">
                <div className="w-20 h-20 bg-gray-800 rounded-full flex items-center justify-center">
                  <div className="w-16 h-16 bg-gray-700 rounded-full flex items-center justify-center">
                    <Upload className="w-8 h-8 text-white" />
                  </div>
                </div>
              </div>
              {/* Animated dots */}
              <div className="absolute inset-0 animate-spin-slow">
                {[...Array(8)].map((_, i) => (
                  <div
                    key={i}
                    className="absolute w-2 h-2 bg-white rounded-full"
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

            <h3 className="text-white mb-2">Importing data</h3>
            <p className="text-gray-400 text-sm text-center mb-6">
              Please wait a few seconds while we&apos;re
              <br />
              importing your data to the project
            </p>

            <Button
              onClick={() => {
                setIsUploading(false);
                onOpenChange(false);
              }}
              className="w-full bg-white hover:bg-gray-100 text-black"
            >
              Cancel
            </Button>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
