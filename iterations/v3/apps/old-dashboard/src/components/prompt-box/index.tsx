// src/components/prompt-box/index.tsx
/**
 * Prompt Box - A component that allows the user to enter a prompt and submit it to the arbiter-orchestrator.
 * Here, you can query, chat, and ask for the agent to plan or perform a task.
 * This functions like a multi-modal message box to add context to the agent's decision-making process.
 * There are several sub-components that make up this component:
 * - PromptInput: A textarea for the user to enter their prompt.
 * - PromptSubmit: A button to submit the prompt to the arbiter-orchestrator with different priority: [now, soon, after]
 * - AttachmentUpload: A button to upload a file to the arbiter-orchestrator.
 * - Menu: A dropdown menu to select the type of prompt to submit: [chat, planning, agent].
 * - ContextList: A collection of context items that the user has added to the prompt's context. These will be displayed as dismissable cards that can be expanded to preview the context.
 * @author @darianrosebrook
 */

'use client';

import styles from './prompt-box.module.scss';
import PromptInput from './PromptInput';
import PromptSubmit from './PromptSubmit';
import AttachmentUpload from './AttachmentUpload';
import Menu from './Menu';
import ContextList from './ContextList';

export default function PromptBox() {
    return (
        <div className={styles.promptBox}>
            <PromptInput />
            <PromptSubmit />
            <AttachmentUpload />
            <Menu />
            <ContextList />
        </div>
    );
}