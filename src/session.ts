import { useEffect, useState } from "react";

const SERVER_URL = "ws://192.168.50.3/ws/";

type OpenHandler = (ev: Event) => void;
type ErrorHandler = (ev: Event) => void;
type MessageHandler = (ev: MessageEvent) => void;
type CloseHandler = (ev: CloseEvent) => void;

export function useSession(
  onOpen: OpenHandler,
  onError: ErrorHandler,
  onMessage: MessageHandler,
  onClose: CloseHandler,
): WebSocket {
  const [session, setSession] =
    useState<WebSocket>(null as unknown as WebSocket);

  if (session === null) {
    setSession(new WebSocket(SERVER_URL));
  }

  const updateOpenHandler = () => {
    if (!session) return;
    session.addEventListener('open', onOpen);
    return () => {
      session.removeEventListener('open', onOpen);
    };
  };

  const updateErrorHandler = () => {
    if (!session) return;
    session.addEventListener('error', onError);
    return () => {
      session.removeEventListener('error', onError);
    };
  };

  const updateMessageHandler = () => {
    if (!session) return;
    session.addEventListener('message', onMessage);
    return () => {
      session.removeEventListener('message', onMessage);
    };
  };

  const updateCloseHandler = () => {
    if (!session) return;
    session.addEventListener('close', onClose);
    return () => {
      session.removeEventListener('close', onClose);
    };
  };

  useEffect(updateOpenHandler, [session, onOpen]);
  useEffect(updateErrorHandler, [session, onError]);
  useEffect(updateMessageHandler, [session, onMessage]);
  useEffect(updateCloseHandler, [session, onClose]);

  return session;
}
