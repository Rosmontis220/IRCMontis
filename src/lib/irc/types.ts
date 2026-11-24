export interface Member {
  nick: string;
  prefix: string;
}

export interface SaslAccount {
  account: string;
  password: string;
}

export interface ConnectConfig {
  nick: string;
  sasl?: SaslAccount | null;
  auto_join?: string[];
  host?: string;
  port?: number;
}

export type IrcEvent =
  | { type: 'connected'; nick: string }
  | { type: 'disconnected'; reason: string; reconnectInSecs: number | null }
  | { type: 'nickInUse'; attempted: string }
  | { type: 'selfNick'; nick: string }
  | { type: 'message'; session: string; sender: string; text: string; kind: 'chat' | 'action' | 'notice'; ts: number; msgid: string | null }
  | { type: 'joined'; channel: string; nick: string }
  | { type: 'parted'; channel: string; nick: string; reason: string | null }
  | { type: 'kicked'; channel: string; nick: string; setter: string; reason: string | null }
  | { type: 'quit'; nick: string; reason: string | null }
  | { type: 'nickChange'; old: string; new: string }
  | { type: 'topic'; channel: string; topic: string }
  | { type: 'names'; channel: string; names: Member[] }
  | { type: 'mode'; channel: string; setter: string; modes: string; args: string[] }
  | { type: 'joinFailed'; channel: string; reason: string }
  | { type: 'sendFailed'; target: string; reason: string }
  | { type: 'serverNotice'; text: string };

export interface ChatMessage {
  id?: number;
  session: string;
  sender: string;
  text: string;
  kind: 'chat' | 'action' | 'notice' | 'system';
  ts: number;
  self?: boolean;
  failed?: boolean;
}

export interface StoredMessage {
  id: number;
  session: string;
  sender: string;
  text: string;
  kind: string;
  ts: number;
  msgid: string | null;
}

export interface SessionRow {
  name: string;
  kind: 'channel' | 'pm';
  last_ts: number;
  muted: boolean;
}