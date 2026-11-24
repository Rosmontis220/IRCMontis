import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ConnectConfig, IrcEvent, SessionRow, StoredMessage } from './types';

export const api = {
  connect: (config: ConnectConfig) => invoke<void>('irc_connect', { config }),
  disconnect: () => invoke<void>('irc_disconnect'),
  send: (target: string, text: string) => invoke<void>('irc_send', { target, text }),
  join: (channel: string) => invoke<void>('irc_join', { channel }),
  part: (channel: string, reason?: string) => invoke<void>('irc_part', { channel, reason }),
  setNick: (nick: string) => invoke<void>('irc_set_nick', { nick }),
  appendMessage: (session: string, sender: string, text: string, kind: string, ts: number) =>
    invoke<number>('message_append', { session, sender, text, kind, ts }),
  history: (session: string, beforeTs?: number, limit?: number) =>
    invoke<StoredMessage[]>('history_load', { session, beforeTs, limit }),
  search: (session: string, query: string, limit?: number) =>
    invoke<StoredMessage[]>('history_search', { session, query, limit }),
  sessions: () => invoke<SessionRow[]>('sessions_list'),
  setMuted: (name: string, muted: boolean) => invoke<void>('session_set_muted', { name, muted }),
  deleteSession: (name: string) => invoke<void>('session_delete', { name }),
  settingGet: (key: string) => invoke<string | null>('setting_get', { key }),
  settingSet: (key: string, value: string) => invoke<void>('setting_set', { key, value }),
  trayUnread: (on: boolean) => invoke<void>('tray_set_unread', { on }),
  trayLabels: (show: string, quit: string) => invoke<void>('tray_set_labels', { show, quit }),
  saveBackground: (path: string) => invoke<string>('save_background', { path }),
  onEvent: (cb: (ev: IrcEvent) => void): Promise<UnlistenFn> =>
    listen<IrcEvent>('irc-event', (e) => cb(e.payload))
};