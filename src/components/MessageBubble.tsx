import { useState, useCallback } from "react";
import type { InnerState, Message } from "../types";
import {
  agentAccent,
  agentInitials,
  agentLabel,
  relativeTime,
} from "../types";
import MarkdownBody from "./MarkdownBody";

interface Props {
  message: Message;
  tick: number;
  config?: InnerState | null;
  showThoughtsUi: boolean;
  onDelete?: (agent: string, turn: number, created_at: number) => void;
}

export default function MessageBubble({
  message,
  tick: _tick,
  config,
  showThoughtsUi,
  onDelete,
}: Props) {
  const [copied, setCopied] = useState(false);
  const [thoughtsOpen, setThoughtsOpen] = useState(false);
  const label = agentLabel(message.agent, config);
  const accent = agentAccent(message.agent);
  const isSeed = message.agent === "seed";
  const isStream = !!message.streaming;
  const reasoning = (message.reasoning || "").trim();
  const hasThoughts = showThoughtsUi && reasoning.length > 0;

  const handleCopy = useCallback(() => {
    void navigator.clipboard.writeText(message.content).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1200);
    });
  }, [message.content]);

  return (
    <article
      className={`msg ${isSeed ? "seed" : ""} ${isStream ? "streaming" : ""}`}
    >
      <div
        className="msg-avatar"
        style={{
          background: isSeed ? "var(--elev)" : accent,
          color: isSeed ? "var(--text-2)" : "#12141a",
        }}
        aria-hidden
      >
        {isSeed ? "You" : agentInitials(label)}
      </div>

      <div className="msg-body">
        <div className="msg-meta">
          <span
            className="msg-name"
            style={{ color: isSeed ? undefined : accent }}
          >
            {label}
          </span>
          {isStream ? (
            <span className="msg-live">live</span>
          ) : (
            <span className="msg-time">
              {relativeTime(message.created_at || Date.now())}
            </span>
          )}
          <button
            type="button"
            className="btn btn-ghost btn-sm msg-copy"
            onClick={handleCopy}
          >
            {copied ? "Copied" : "Copy"}
          </button>
        </div>

        {hasThoughts && (
          <div className="thoughts">
            <button
              type="button"
              className={`thoughts-toggle ${thoughtsOpen ? "open" : ""}`}
              onClick={() => setThoughtsOpen((o) => !o)}
              aria-expanded={thoughtsOpen}
            >
              <span className="thoughts-chevron">
                {thoughtsOpen ? "▾" : "▸"}
              </span>
              <span className="thoughts-label">Thoughts</span>
              {isStream && !message.content && (
                <span className="msg-live" style={{ marginLeft: 6 }}>
                  thinking…
                </span>
              )}
            </button>
            {thoughtsOpen && (
              <div className="thoughts-body">
                <MarkdownBody
                  content={reasoning}
                  streaming={isStream && !message.content}
                />
              </div>
            )}
          </div>
        )}

        {(message.content || !hasThoughts) && (
          <div className="msg-bubble">
            <MarkdownBody content={message.content} streaming={isStream} />
          </div>
        )}

        {!isStream && onDelete && (
          <button
            type="button"
            className="msg-del"
            title="Delete message"
            aria-label="Delete message"
            onClick={() =>
              onDelete(message.agent, message.turn, message.created_at)
            }
          >
            Delete
          </button>
        )}
      </div>
    </article>
  );
}
