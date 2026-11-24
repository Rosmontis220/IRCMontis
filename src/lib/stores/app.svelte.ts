import { get } from 'svelte/store';
import { _ } from 'svelte-i18n';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { platform } from '@tauri-apps/plugin-os';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import { api } from '$lib/irc/api';
import type { ChatMessage, IrcEvent, Member } from '$lib/irc/types';
import { plainText } from '$lib/format';
import { setupI18n, setLocale } from '$lib/i18n';
import { applyTheme, applySystemTheme } from '$lib/theme';

export interface Settings {
  locale: 'system' | 'zh' | 'en';
  theme: 'light' | 'dark' | 'system';
  backgroundPath: string | null;
  backgroundOpacity: number;
  closeToTray: boolean;
  notifications: boolean;
  nick: string;
  saslAccount: string;
  saslPassword: string;
}

const defaultSettings: Settings = {
  locale: 'system',
  theme: 'system',
  backgroundPath: null,
  backgroundOpacity: 30,
  closeToTray: true,
  notifications: true,
  nick: '',
  saslAccount: '',
  saslPassword: ''
};

export interface Session {
  name: string;
  kind: 'channel' | 'pm';
  lastTs: number;
  muted: boolean;
  unread: number;
  mention: boolean;
  topic: string;
  members: Member[];
}

const tr = (key: string, values?: Record<string, string | number>) => get(_)(key, { values });

class AppStore {
  settings = $state<Settings>({ ...defaultSettings });
  connected = $state(false);
  connecting = $state(false);
  nick = $state('');
  reconnectIn = $state<number | null>(null);
  statusReason = $state('');
  nickInUse = $state<string | null>(null);
  sessions = $state<Session[]>([]);
  active = $state<string | null>(null);
  messages = $state<Record<string, ChatMessage[]>>({});
  loaded = $state<Record<string, boolean>>({});
  focused = $state(true);
  ready = $state(false);
  showConnect = $state(false);
  showSettings = $state(false);
  showJoin = $state(false);

  async init() {
    const raw = await api.settingGet('settings');
    if (raw) {
      try {
        Object.assign(this.settings, JSON.parse(raw));
      } catch {
        // corrupted settings blob: fall back to defaults
      }
    }
    setupI18n(this.settings.locale);
    this.applyAppearance();
    matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if (this.settings.theme === 'system') this.applyAppearance();
    });
    const rows = await api.sessions();
    this.sessions = rows.map((r) => ({
      name: r.name,
      kind: r.kind,
      lastTs: r.last_ts,
      muted: r.muted,
      unread: 0,
      mention: false,
      topic: '',
      members: []
    }));
    await api.onEvent((ev) => this.onEvent(ev));
    try {
      getCurrentWindow().onFocusChanged(({ payload }) => {
        this.focused = payload;
        this.syncTray();
      });
      // hide()/show() bypass WebView2 focus events; Rust emits these explicitly
      void getCurrentWindow().listen('window-hidden', () => {
        this.focused = false;
        this.syncTray();
      });
      void getCurrentWindow().listen('window-shown', () => {
        this.focused = true;
        this.syncTray();
      });
    } catch {
      // plain-browser dev mode
    }
    if (this.settings.nick) {
      this.connect();
    } else {
      this.showConnect = true;
    }
    this.ready = true;
  }

  persist() {
    void api.settingSet('settings', JSON.stringify(this.settings));
    void api.settingSet('close_to_tray', String(this.settings.closeToTray));
  }

  applyAppearance() {
    if (this.settings.theme === 'system') void applySystemTheme();
    else applyTheme(this.settings.theme);
  }

  connect() {
    if (!this.settings.nick.trim()) {
      this.showConnect = true;
      return;
    }
    this.connecting = true;
    this.reconnectIn = null;
    this.showConnect = false;
    void api.connect({
      nick: this.settings.nick.trim(),
      sasl: this.settings.saslAccount ? { account: this.settings.saslAccount, password: this.settings.saslPassword } : null,
      auto_join: this.sessions.filter((s) => s.kind === 'channel').map((s) => s.name)
    });
    this.syncKeepAlive(true);
  }

  disconnect() {
    void api.disconnect();
    this.syncKeepAlive(false);
  }

  syncKeepAlive(on: boolean) {
    try {
      if (platform() === 'android') {
        void invoke(on ? 'plugin:background-service|start' : 'plugin:background-service|stop').catch(() => {});
      }
    } catch {
      // plain-browser dev mode
    }
  }

  session(name: string): Session | undefined {
    return this.sessions.find((s) => s.name === name);
  }

  ensureSession(name: string, kind: 'channel' | 'pm'): Session {
    let s = this.session(name);
    if (!s) {
      s = { name, kind, lastTs: 0, muted: false, unread: 0, mention: false, topic: '', members: [] };
      this.sessions.push(s);
    }
    return s;
  }

  sortSessions() {
    this.sessions.sort((a, b) => b.lastTs - a.lastTs);
  }

  pushMessage(session: string, msg: ChatMessage) {
    // read back after assign: $state proxy stores a wrapped copy, the ??= rhs would be detached
    if (!this.messages[session]) this.messages[session] = [];
    const arr = this.messages[session];
    arr.push(msg);
    if (arr.length > 800) arr.splice(0, arr.length - 800);
  }

  pushSystem(session: string, text: string) {
    const ts = Date.now();
    this.pushMessage(session, { session, sender: '', text, kind: 'system', ts });
    void api.appendMessage(session, '', text, 'system', ts);
  }

  openSession(name: string, kind: 'channel' | 'pm' = 'pm') {
    const s = this.ensureSession(name, kind);
    this.active = name;
    s.unread = 0;
    s.mention = false;
    this.syncTray();
    if (!this.loaded[name]) void this.loadHistory(name);
  }

  async loadHistory(session: string) {
    const rows = await api.history(session, undefined, 100);
    const mapped = rows.map((r) => ({
      id: r.id,
      session: r.session,
      sender: r.sender,
      text: r.text,
      kind: r.kind as ChatMessage['kind'],
      ts: r.ts,
      self: r.sender === this.nick && r.kind !== 'system'
    }));
    // keep live messages that arrived after the newest stored row (or not yet committed)
    const lastTs = mapped.at(-1)?.ts ?? 0;
    const live = (this.messages[session] ?? []).filter((m) => m.ts > lastTs);
    this.messages[session] = [...mapped, ...live];
    this.loaded[session] = true;
  }

  async loadOlder(session: string): Promise<number> {
    const first = this.messages[session]?.[0];
    if (!first) return 0;
    const rows = await api.history(session, first.ts, 100);
    const older = rows.map((r) => ({
      id: r.id,
      session: r.session,
      sender: r.sender,
      text: r.text,
      kind: r.kind as ChatMessage['kind'],
      ts: r.ts,
      self: r.sender === this.nick && r.kind !== 'system'
    }));
    this.messages[session] = [...older, ...(this.messages[session] ?? [])];
    return older.length;
  }

  async jumpTo(session: string, ts: number): Promise<number> {
    const rows = await api.history(session, ts + 1, 100);
    this.messages[session] = rows.map((r) => ({
      id: r.id,
      session: r.session,
      sender: r.sender,
      text: r.text,
      kind: r.kind as ChatMessage['kind'],
      ts: r.ts,
      self: r.sender === this.nick && r.kind !== 'system'
    }));
    this.loaded[session] = true;
    return rows.length;
  }

  sendTo(target: string, text: string, kind: 'chat' | 'action') {
    const ts = Date.now();
    this.pushMessage(target, { session: target, sender: this.nick, text, kind, ts, self: true });
    void api.appendMessage(target, this.nick, text, kind, ts);
    void api.send(target, text);
    const s = this.session(target);
    if (s) {
      s.lastTs = ts;
      this.sortSessions();
    }
  }

  sendInput(raw: string) {
    const text = raw.trim();
    if (!text || !this.active) return;
    if (text.startsWith('/')) {
      const [cmd, ...restArgs] = text.slice(1).split(' ');
      const rest = restArgs.join(' ');
      switch (cmd.toLowerCase()) {
        case 'me':
          if (rest) this.sendTo(this.active, rest, 'action');
          return;
        case 'join':
          if (rest) void api.join(rest);
          return;
        case 'part':
          void api.part(rest || this.active);
          return;
        case 'nick':
          if (rest) void api.setNick(rest);
          return;
        case 'msg': {
          const [target, ...msgParts] = restArgs;
          const body = msgParts.join(' ');
          if (target && body) {
            this.openSession(target, 'pm');
            this.sendTo(target, body, 'chat');
          }
          return;
        }
        default:
          this.pushSystem(this.active, tr('system.unknownCommand', { cmd }));
          return;
      }
    }
    this.sendTo(this.active, text, 'chat');
  }

  async setMuted(name: string, muted: boolean) {
    const s = this.session(name);
    if (s) s.muted = muted;
    await api.setMuted(name, muted);
  }

  async leaveChannel(name: string) {
    await api.part(name);
    await api.deleteSession(name);
    this.sessions = this.sessions.filter((s) => s.name !== name);
    delete this.messages[name];
    delete this.loaded[name];
    if (this.active === name) this.active = null;
  }

  async deleteSession(name: string) {
    await api.deleteSession(name);
    this.sessions = this.sessions.filter((s) => s.name !== name);
    delete this.messages[name];
    delete this.loaded[name];
    if (this.active === name) this.active = null;
  }

  syncTray() {
    const unread = this.sessions.some((s) => s.unread > 0);
    void api.trayUnread(unread && !this.focused);
  }

  async notify(session: string, sender: string, text: string) {
    if (!this.settings.notifications) return;
    if (this.session(session)?.muted) return;
    if (this.focused && this.active === session) return;
    try {
      let granted = await isPermissionGranted();
      if (!granted) granted = (await requestPermission()) === 'granted';
      if (!granted) return;
      const s = this.session(session);
      sendNotification({
        title: s?.kind === 'channel' ? `${sender} · ${session}` : sender,
        body: plainText(text).slice(0, 120)
      });
    } catch {
      // notification subsystem unavailable
    }
  }

  onEvent(ev: IrcEvent) {
    switch (ev.type) {
      case 'connected':
        this.connected = true;
        this.connecting = false;
        this.nick = ev.nick;
        this.settings.nick = ev.nick;
        this.reconnectIn = null;
        this.persist();
        this.pushSystem('*server', tr('system.connected'));
        break;
      case 'disconnected':
        this.connected = false;
        this.connecting = false;
        this.reconnectIn = ev.reconnectInSecs;
        this.statusReason = ev.reason;
        if (ev.reason !== 'manual') this.pushSystem('*server', tr('system.disconnected', { reason: ev.reason }));
        break;
      case 'nickInUse':
        this.connecting = false;
        this.nickInUse = ev.attempted;
        break;
      case 'selfNick':
        this.nick = ev.nick;
        this.settings.nick = ev.nick;
        this.persist();
        break;
      case 'message': {
        const kind = ev.session.startsWith('#') || ev.session.startsWith('&') ? 'channel' : 'pm';
        const s = this.ensureSession(ev.session, kind);
        this.pushMessage(ev.session, { session: ev.session, sender: ev.sender, text: ev.text, kind: ev.kind, ts: ev.ts });
        s.lastTs = ev.ts;
        if (this.active !== ev.session || !this.focused) s.unread += 1;
        if (this.nick && ev.text.toLowerCase().includes(this.nick.toLowerCase())) s.mention = true;
        this.sortSessions();
        void this.notify(ev.session, ev.sender, ev.text);
        break;
      }
      case 'joined': {
        const s = this.ensureSession(ev.channel, 'channel');
        if (ev.nick === this.nick) {
          this.pushSystem(ev.channel, tr('system.joined', { nick: ev.nick }));
        } else {
          if (!s.members.some((m) => m.nick === ev.nick)) s.members.push({ nick: ev.nick, prefix: '' });
          this.pushSystem(ev.channel, tr('system.joined', { nick: ev.nick }));
        }
        break;
      }
      case 'parted': {
        const s = this.session(ev.channel);
        if (s) {
          s.members = s.members.filter((m) => m.nick !== ev.nick);
          this.pushSystem(ev.channel, tr('system.parted', { nick: ev.nick }));
        }
        break;
      }
      case 'kicked': {
        const s = this.session(ev.channel);
        if (s) {
          s.members = s.members.filter((m) => m.nick !== ev.nick);
          if (ev.nick === this.nick) s.members = [];
          this.pushSystem(ev.channel, tr('system.kicked', { nick: ev.nick, setter: ev.setter }));
        }
        break;
      }
      case 'quit':
        for (const s of this.sessions) {
          if (s.kind === 'channel' && s.members.some((m) => m.nick === ev.nick)) {
            s.members = s.members.filter((m) => m.nick !== ev.nick);
            this.pushSystem(s.name, tr('system.quit', { nick: ev.nick }));
          }
        }
        break;
      case 'nickChange':
        for (const s of this.sessions) {
          if (s.members.some((m) => m.nick === ev.old)) {
            s.members = s.members.map((m) => (m.nick === ev.old ? { ...m, nick: ev.new } : m));
            this.pushSystem(s.name, tr('system.nickChange', { old: ev.old, new: ev.new }));
          }
        }
        break;
      case 'topic': {
        const s = this.session(ev.channel);
        if (s) s.topic = ev.topic;
        break;
      }
      case 'names': {
        const s = this.session(ev.channel);
        if (s) s.members = ev.names;
        break;
      }
      case 'mode':
        this.pushSystem(ev.channel, tr('system.mode', { setter: ev.setter, modes: `${ev.modes} ${ev.args.join(' ')}`.trim() }));
        break;
      case 'joinFailed':
        this.pushSystem(this.session(ev.channel) ? ev.channel : '*server', tr('system.joinFailed', { channel: ev.channel, reason: ev.reason }));
        break;
      case 'sendFailed': {
        const arr = this.messages[ev.target];
        if (arr) {
          for (let i = arr.length - 1; i >= 0; i--) {
            if (arr[i].self) {
              arr[i].failed = true;
              break;
            }
          }
        }
        this.pushSystem(this.session(ev.target) ? ev.target : '*server', tr('system.sendFailed', { reason: ev.reason }));
        break;
      }
      case 'serverNotice':
        this.pushSystem('*server', ev.text);
        break;
    }
    this.syncTray();
  }
}

export const app = new AppStore();