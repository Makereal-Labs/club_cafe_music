import { useEffect, useState } from "react";

const SERVER_URL = "wss://pi.makereallabs.org/ws/";

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

  const _onClose = (ev: CloseEvent) => {
    setTimeout(() => {
      if (session.readyState == session.CLOSED) {
        setSession(new WebSocket(SERVER_URL));
      }
    }, 100);
    onClose(ev);
  };

  const updateCloseHandler = () => {
    if (!session) return;
    session.addEventListener('close', _onClose);
    return () => {
      session.removeEventListener('close', _onClose);
    };
  };

  useEffect(updateOpenHandler, [session, onOpen]);
  useEffect(updateErrorHandler, [session, onError]);
  useEffect(updateMessageHandler, [session, onMessage]);
  useEffect(updateCloseHandler, [session, onClose]);

  return session;
}
