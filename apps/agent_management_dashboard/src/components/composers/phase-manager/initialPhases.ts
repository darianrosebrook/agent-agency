import type { Phase } from './types';

export const initialPhases: Phase[] = [
  {
    id: 'phase-1',
    number: 1,
    title: 'Research & Planning',
    description:
      'Understand the requirements and plan the architecture for a multi-modal RAG search UI tool.',
    tasks: [
      {
        id: 'task-1',
        title: 'Define core features',
        description:
          'Identify the key features needed for multi-modal RAG (Retrieval Augmented Generation) including text, image, and vector search capabilities.',
        subtasks: [],
        contextChips: [],
      },
      {
        id: 'task-2',
        title: 'Research vector databases',
        description:
          'Evaluate options like Pinecone, Weaviate, and Qdrant for storing and querying embeddings.',
        subtasks: [],
        contextChips: [],
      },
      {
        id: 'task-3',
        title: 'Design UI/UX wireframes',
        description:
          'Create mockups for the search interface, results display, and filter controls.',
        subtasks: [],
        contextChips: [],
      },
    ],
  },
  {
    id: 'phase-2',
    number: 2,
    title: 'Foundation Setup',
    description: 'Set up the development environment and core infrastructure.',
    tasks: [
      {
        id: 'task-4',
        title: 'Set up integrations',
        description:
          'Configure API connections for vector database, embedding models, and LLM providers.',
        subtasks: [],
        contextChips: [],
      },
      {
        id: 'task-5',
        title: 'Initialize project structure',
        description:
          'Set up the repository with TypeScript, React, and necessary build tools.',
        subtasks: [],
        contextChips: [],
      },
    ],
  },
];

