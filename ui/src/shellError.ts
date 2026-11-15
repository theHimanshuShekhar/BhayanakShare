// Errors the shell gives as `{ kind, message }`: how each is worded.

import { t, type MessageKey } from "./i18n";

/**
 * The catalogue message in `messages` for an error the shell gives as `{ kind, message }`, with
 * the message as its `{reason}`. Anything else (a kind this UI does not know, or a failure of the
 * call itself) is worded `other`, with its text as the reason.
 */
export function wordShellError<K extends string>(e: unknown, messages: Record<K, MessageKey>, other: MessageKey): string {
  if (typeof e === "object" && e !== null && "kind" in e && "message" in e && Object.hasOwn(messages, String(e.kind))) {
    return t(messages[e.kind as K], { reason: String(e.message) });
  }
  return t(other, { reason: e instanceof Error ? e.message : String(e) });
}
