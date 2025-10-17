import { useState } from "react";

interface ObservationFormProps {
  onSubmit: (message: string, author?: string) => Promise<void>;
  onCancel: () => void;
}

export default function ObservationForm({
  onSubmit,
  onCancel,
}: ObservationFormProps) {
  const [message, setMessage] = useState("");
  const [author, setAuthor] = useState("");
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!message.trim()) return;

    setSubmitting(true);
    try {
      await onSubmit(message.trim(), author.trim() || undefined);
      setMessage("");
      setAuthor("");
    } catch (error) {
      console.error("Failed to submit observation:", error);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="border border-gray-200 rounded-lg p-4 bg-gray-50">
      <h4 className="text-sm font-medium text-gray-900 mb-3">
        Add Observation
      </h4>
      <form onSubmit={handleSubmit} className="space-y-3">
        <div>
          <label
            htmlFor="observation-message"
            className="block text-xs font-medium text-gray-700 mb-1"
          >
            Message
          </label>
          <textarea
            id="observation-message"
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            placeholder="Enter your observation or note..."
            className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm resize-none"
            rows={3}
            required
          />
        </div>

        <div>
          <label
            htmlFor="observation-author"
            className="block text-xs font-medium text-gray-700 mb-1"
          >
            Author (Optional)
          </label>
          <input
            type="text"
            id="observation-author"
            value={author}
            onChange={(e) => setAuthor(e.target.value)}
            placeholder="Your name or identifier"
            className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm"
          />
        </div>

        <div className="flex justify-end space-x-2">
          <button
            type="button"
            onClick={onCancel}
            className="px-3 py-1.5 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={submitting || !message.trim()}
            className="px-3 py-1.5 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {submitting ? "Adding..." : "Add Observation"}
          </button>
        </div>
      </form>
    </div>
  );
}
