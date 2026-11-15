// Offering files: how files the Device could not offer are worded.

import type { SendError, SendErrorKind } from "./bindings";
import { t, type MessageKey } from "./i18n";

/**
 * The catalogue message for each reason files could not be offered: C8 for a selection over the
 * Offer's limits, else the reason in brackets. A kind added in the shell fails the typecheck
 * until it is worded.
 */
export const SEND_ERRORS: Record<SendErrorKind, MessageKey> = {
  too_big: "send.tooBig",
  other: "send.failed",
};

function isSendError(e: unknown): e is SendError {
  return typeof e === "object" && e !== null && "kind" in e && "message" in e;
}

/**
 * Why an Offer could not be sent, in words for the person using the app. `other` words a reason
 * that is not one of the catalogue's own, with the reason in brackets: starting a Transfer again
 * says so.
 */
export function sendFailure(e: unknown, other: MessageKey = SEND_ERRORS.other): string {
  const [kind, reason]: [SendErrorKind, string] = isSendError(e)
    ? [e.kind, e.message]
    : ["other", e instanceof Error ? e.message : String(e)];
  return t(kind === "other" ? other : SEND_ERRORS[kind], { reason });
}
