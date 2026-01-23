"use client";

import React from "react";
import { motion } from "framer-motion";
import { Clock, Target, Users } from "lucide-react";
import { cn } from "@/lib/utils";

export interface Project {
  id: string;
  title: string;
  description: string;
  category: "Tech" | "Art" | "Green Energy" | "UX";
  goal: number;
  raised: number;
  backers: number;
  daysLeft: number;
  imageUrl: string;
  createdAt: string;
}

interface ProjectCardProps {
  project: Project;
}

export const ProjectCard: React.FC<ProjectCardProps> = ({ project }) => {
  const progress = Math.min((project.raised / project.goal) * 100, 100);

  const categoryColors = {
    Tech: "bg-blue-500/10 text-blue-500 border-blue-500/20",
    Art: "bg-purple-500/10 text-purple-500 border-purple-500/20",
    "Green Energy": "bg-emerald-500/10 text-emerald-500 border-emerald-500/20",
    UX: "bg-orange-500/10 text-orange-500 border-orange-500/20",
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      whileHover={{ y: -5 }}
      transition={{ duration: 0.4, ease: "easeOut" }}
      className="group relative flex flex-col overflow-hidden rounded-2xl border border-white/10 bg-white/5 transition-colors hover:border-white/20 hover:bg-white/10"
    >
      {/* Project Image Placeholder */}
      <div className="relative aspect-video w-full overflow-hidden bg-white/5">
        {project.imageUrl ? (
          <img 
            src={project.imageUrl} 
            alt={project.title} 
            className="h-full w-full object-cover transition-transform duration-500 group-hover:scale-110"
          />
        ) : (
          <div 
            className="absolute inset-0 bg-gradient-to-br from-primary/20 to-transparent" 
            style={{ opacity: 0.5 }}
          />
        )}
        <div className="absolute inset-0 flex items-center justify-center text-white/20">
          {!project.imageUrl && <Target className="h-12 w-12" />}
        </div>
        <div className="absolute right-4 top-4">
          <span
            className={cn(
              "px-3 py-1 rounded-full text-xs font-medium border backdrop-blur-md",
              categoryColors[project.category],
            )}
          >
            {project.category}
          </span>
        </div>
      </div>

      <div className="flex flex-1 flex-col p-6">
        <h3 className="line-clamp-1 text-xl font-bold text-white transition-colors group-hover:text-primary">
          {project.title}
        </h3>
        <p className="mt-2 line-clamp-2 text-sm text-white/60">
          {project.description}
        </p>

        <div className="mt-8 space-y-4">
          {/* Progress Bar Container */}
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span className="font-medium text-white">
                ${project.raised.toLocaleString()}
              </span>
              <span className="text-white/40">
                {Math.round(progress)}% of ${project.goal.toLocaleString()}
              </span>
            </div>
            <div className="h-2 w-full overflow-hidden rounded-full bg-white/10">
              <motion.div
                initial={{ width: 0 }}
                animate={{ width: `${progress}%` }}
                transition={{ duration: 1, ease: "easeOut", delay: 0.2 }}
                className="h-full bg-primary"
              />
            </div>
          </div>

          {/* Stats */}
          <div className="flex items-center justify-between border-t border-white/5 pt-4 text-sm text-white/40">
            <div className="flex items-center gap-1.5">
              <Users className="h-4 w-4" />
              <span>{project.backers} backers</span>
            </div>
            <div className="flex items-center gap-1.5">
              <Clock className="h-4 w-4" />
              <span>{project.daysLeft}d left</span>
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  );
};
