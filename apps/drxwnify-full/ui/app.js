/**
 * Drxwnify Full Edition — Frontend Application Logic
 */

(function () {
  // Tauri IPC helper (Dynamic bridge to prevent evaluation before injection)
  const invoke = async (cmd, args = {}) => {
    try {
      if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === "function") {
        return await window.__TAURI_INTERNALS__.invoke(cmd, args);
      }
      if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === "function") {
        return await window.__TAURI__.core.invoke(cmd, args);
      }
      if (window.__TAURI__ && typeof window.__TAURI__.invoke === "function") {
        return await window.__TAURI__.invoke(cmd, args);
      }
      console.warn(`[IPC Fallback] ${cmd}`, args);
      return null;
    } catch (err) {
      console.error(`[IPC Error: ${cmd}]`, err);
      throw err;
    }
  };

  // State
  const state = {
    currentScreen: "home",
    historyStack: ["home"],
    historyIndex: 0,
    isPlaying: false,
    currentTrack: null,
    positionMs: 0,
    durationMs: 0,
    volume: 0.8,
    isMuted: false,
    repeatMode: "off",
    isShuffled: false,
    queue: [],
    queueIndex: 0,
    likedTracks: [],
    playlists: [],
    history: [],
    friends: [],
    activeChatFriend: null,
    chatMessages: {},
    lyrics: null,
    translatedLyrics: null,
    showTranslation: false,
    theme: "tategaki",
    mode: "night",
    look: "glass",
  };

  // DOM Elements
  const els = {
    body: document.body,
    brandLogo: document.getElementById("brand-logo"),
    navHomeBtn: document.getElementById("nav-home-btn"),
    btnBack: document.getElementById("nav-back"),
    btnForward: document.getElementById("nav-forward"),
    btnRefresh: document.getElementById("nav-refresh"),
    searchInput: document.getElementById("search-input"),
    searchClear: document.getElementById("search-clear"),
    btnTopFriends: document.getElementById("btn-top-friends"),
    btnTopIntegrations: document.getElementById("btn-top-integrations"),
    btnTopDownloads: document.getElementById("btn-top-downloads"),
    btnPhoneSync: document.getElementById("btn-phone-sync"),
    btnThemeSelector: document.getElementById("btn-theme-selector"),
    modalTheme: document.getElementById("modal-theme"),
    modalThemeClose: document.getElementById("modal-theme-close"),
    modalSleepTimer: document.getElementById("modal-sleep-timer"),
    modalSleepClose: document.getElementById("modal-sleep-close"),
    modalPhoneSync: document.getElementById("modal-phone-sync"),
    modalBackupClose: document.getElementById("modal-backup-close"),
    modalNewPlaylist: document.getElementById("modal-new-playlist"),
    modalPlaylistClose: document.getElementById("modal-playlist-close"),
    btnMin: document.getElementById("btn-minimize"),
    btnMax: document.getElementById("btn-maximize"),
    btnClose: document.getElementById("btn-close"),
    leftPanel: document.getElementById("left-panel"),
    btnCollapseLeft: document.getElementById("btn-collapse-left"),
    btnNewPlaylist: document.getElementById("btn-new-playlist"),
    navItems: document.querySelectorAll(".lib-item, .nav-item"),
    likedCount: document.getElementById("liked-count"),
    navPlaylistsList: document.getElementById("nav-playlists-list"),
    centerContent: document.getElementById("center-content"),
    playerCover: document.getElementById("player-cover"),
    playerTitle: document.getElementById("player-title"),
    playerArtist: document.getElementById("player-artist"),
    playerBtnLike: document.getElementById("player-btn-like"),
    btnShuffle: document.getElementById("btn-shuffle"),
    btnPrev: document.getElementById("btn-prev"),
    btnPlayPause: document.getElementById("btn-play-pause"),
    iconPlay: document.getElementById("icon-play"),
    iconPause: document.getElementById("icon-pause"),
    btnNext: document.getElementById("btn-next"),
    btnRepeat: document.getElementById("btn-repeat"),
    repeatBadge: document.getElementById("repeat-badge"),
    timeCurrent: document.getElementById("time-current"),
    timeDuration: document.getElementById("time-duration"),
    seekBarTrack: document.getElementById("seek-bar-track"),
    seekBarFill: document.getElementById("seek-bar-fill"),
    seekBarThumb: document.getElementById("seek-bar-thumb"),
    volBarTrack: document.getElementById("volume-bar-track"),
    volBarFill: document.getElementById("volume-bar-fill"),
    volBarThumb: document.getElementById("volume-bar-thumb"),
    btnMute: document.getElementById("btn-mute"),
    levelsCanvas: document.getElementById("levels-canvas"),
    btnSleepTimer: document.getElementById("btn-sleep-timer"),
    btnToggleLyrics: document.getElementById("btn-toggle-lyrics"),
    btnToggleQueue: document.getElementById("btn-toggle-queue"),
    btnToggleNowplaying: document.getElementById("btn-toggle-nowplaying"),
    rightPanel: document.getElementById("right-panel"),
    rightTabs: document.querySelectorAll(".tab-btn"),
    tabLyricsContent: document.getElementById("tab-lyrics-content"),
    tabQueueContent: document.getElementById("tab-queue-content"),
    tabInfoContent: document.getElementById("tab-info-content"),
    lyricsLines: document.getElementById("lyrics-lines"),
    toggleTranslate: document.getElementById("toggle-translate-lyrics"),
    queueTracksList: document.getElementById("queue-tracks-list"),
    btnClearQueue: document.getElementById("btn-clear-queue"),
    nowPlayingLargeArt: document.getElementById("now-playing-large-art"),
    nowPlayingDetails: document.getElementById("now-playing-details"),
    polyTrack: document.getElementById("poly-track"),
    polyVol: document.getElementById("poly-vol"),
  };

  // Format mm:ss
  function formatTime(seconds) {
    if (isNaN(seconds) || seconds < 0) return "0:00";
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s < 10 ? "0" : ""}${s}`;
  }

  // --- Theme Management ---
  function applyTheme(theme, mode, look) {
    state.theme = theme;
    state.mode = mode;
    state.look = look || state.look;
    els.body.className = `theme-${state.theme} theme-${state.mode} look-${state.look}`;
    if (window.CanvasBackground) {
      window.CanvasBackground.setTheme(state.theme);
    }
  }

  // --- Window Controls ---
  if (els.btnMin) els.btnMin.onclick = () => invoke("minimize_window");
  if (els.btnMax) els.btnMax.onclick = () => invoke("maximize_window");
  if (els.btnClose) els.btnClose.onclick = () => invoke("close_window");

  // --- Navigation & Screens ---
  function navigateTo(screenId, pushHistory = true) {
    state.currentScreen = screenId;
    if (pushHistory) {
      state.historyStack = state.historyStack.slice(0, state.historyIndex + 1);
      state.historyStack.push(screenId);
      state.historyIndex = state.historyStack.length - 1;
    }

    document.querySelectorAll(".lib-item, .nav-item").forEach((item) => {
      if (item.dataset.screen === screenId) item.classList.add("active");
      else item.classList.remove("active");
    });

    if (els.navHomeBtn) {
      if (screenId === "home") els.navHomeBtn.classList.add("active");
      else els.navHomeBtn.classList.remove("active");
    }

    document.querySelectorAll(".screen-view").forEach((s) => {
      s.classList.add("hidden");
      s.classList.remove("active");
    });
    let target = document.getElementById(`screen-${screenId}`);
    if (!target) {
      target = document.getElementById("screen-home");
    }
    if (target) {
      target.classList.remove("hidden");
      target.classList.add("active");
      renderScreen(screenId, target);
    }
  }

  // Top navigation chevrons & buttons
  if (els.brandLogo) els.brandLogo.onclick = () => navigateTo("home");
  if (els.navHomeBtn) els.navHomeBtn.onclick = () => navigateTo("home");
  if (els.btnTopFriends) els.btnTopFriends.onclick = () => navigateTo("friends");
  if (els.btnTopDownloads) els.btnTopDownloads.onclick = () => navigateTo("downloads");
  if (els.btnTopIntegrations) els.btnTopIntegrations.onclick = () => navigateTo("settings");

  if (els.btnBack) {
    els.btnBack.onclick = () => {
      if (state.historyIndex > 0) {
        state.historyIndex--;
        navigateTo(state.historyStack[state.historyIndex], false);
      }
    };
  }
  if (els.btnForward) {
    els.btnForward.onclick = () => {
      if (state.historyIndex < state.historyStack.length - 1) {
        state.historyIndex++;
        navigateTo(state.historyStack[state.historyIndex], false);
      }
    };
  }
  if (els.btnRefresh) {
    els.btnRefresh.onclick = () => {
      refreshData();
    };
  }

  // Left panel item clicks
  document.querySelectorAll(".lib-item, .nav-item").forEach((item) => {
    item.addEventListener("click", () => {
      const scr = item.dataset.screen;
      if (scr) navigateTo(scr);
    });
  });

  // Left panel collapse toggle
  if (els.btnCollapseLeft && els.leftPanel) {
    els.btnCollapseLeft.onclick = () => {
      els.leftPanel.classList.toggle("collapsed");
    };
  }

  // Library filter chips
  document.querySelectorAll(".library-filter-chips .chip").forEach((chip) => {
    chip.onclick = () => {
      document.querySelectorAll(".library-filter-chips .chip").forEach((c) => c.classList.remove("active"));
      chip.classList.add("active");
      const filter = chip.dataset.filter;
      filterLibraryItems(filter);
    };
  });

  function filterLibraryItems(filter) {
    const items = document.querySelectorAll(".library-items-list .lib-item");
    items.forEach((item) => {
      const scr = item.dataset.screen;
      if (filter === "all") {
        item.style.display = "flex";
      } else if (filter === "playlists") {
        item.style.display = (scr === "liked" || scr === "downloads") ? "flex" : "none";
      } else if (filter === "albums") {
        item.style.display = (scr === "reviews") ? "flex" : "none";
      } else if (filter === "artists") {
        item.style.display = (scr === "friends") ? "flex" : "none";
      }
    });
    const sub = document.getElementById("nav-playlists-list");
    if (sub) {
      sub.style.display = (filter === "all" || filter === "playlists") ? "block" : "none";
    }
  }

  // --- Screens Rendering ---
  function renderScreen(screenId, container) {
    switch (screenId) {
      case "home":
        renderHomeScreen(container);
        break;
      case "liked":
        renderLikedScreen(container);
        break;
      case "search":
        renderSearchScreen(container);
        break;
      case "downloads":
        renderDownloadsScreen(container);
        break;
      case "history":
        renderHistoryScreen(container);
        break;
      case "friends":
        renderFriendsScreen(container);
        break;
      case "reviews":
        renderReviewsScreen(container);
        break;
      case "settings":
        renderSettingsScreen(container);
        break;
      default:
        container.innerHTML = `<div class="empty-screen-msg">Раздел "${screenId}"</div>`;
        break;
    }
  }

  function renderHomeScreen(container) {
    container.innerHTML = `
      <div class="screen-content">
        <div class="home-hero">
          <h1 class="hero-title">Добро пожаловать в Drxwnify</h1>
          <p class="hero-subtitle">Музыкальное полотно с Hi-Fi звучанием, Nostr-сообществом и чистым звуковым ядром.</p>
        </div>
        
        <div class="section-block">
          <div class="section-title">Быстрый доступ</div>
          <div class="quick-picks-grid" id="quick-picks-grid" style="display:grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap:12px; margin-top:10px;">
            <div class="pick-card chip" data-action="liked" style="padding:14px; display:flex; align-items:center; gap:10px; cursor:pointer;">
              <span style="font-size:18px; color:var(--accent);">♥</span>
              <span style="font-weight:700;">Любимые треки</span>
            </div>
            <div class="pick-card chip" data-action="history" style="padding:14px; display:flex; align-items:center; gap:10px; cursor:pointer;">
              <span style="font-size:18px;">⏱</span>
              <span style="font-weight:700;">История</span>
            </div>
            <div class="pick-card chip" data-action="downloads" style="padding:14px; display:flex; align-items:center; gap:10px; cursor:pointer;">
              <span style="font-size:18px;">⬇</span>
              <span style="font-weight:700;">Загрузки</span>
            </div>
            <div class="pick-card chip" data-action="settings" style="padding:14px; display:flex; align-items:center; gap:10px; cursor:pointer;">
              <span style="font-size:18px;">⚙</span>
              <span style="font-weight:700;">Настройки</span>
            </div>
          </div>
        </div>

        <div class="section-block" style="margin-top:24px;">
          <div class="section-title">Недавно играли</div>
          <div class="recent-tracks-list" id="home-recent-tracks" style="display:flex; flex-direction:column; gap:6px; margin-top:10px;">
            ${
              state.history.length === 0
                ? '<div class="empty-notice">Здесь появятся ваши недавно воспроизведённые треки.</div>'
                : state.history
                    .slice(0, 10)
                    .map(
                      (p) => `
              <div class="track-row" data-id="${p.track.id}" style="display:flex; align-items:center; justify-content:space-between; padding:8px 12px; background:var(--fill); border-radius:var(--cover-radius); cursor:pointer;">
                <div class="track-left" style="display:flex; align-items:center; gap:12px;">
                  <div class="track-thumb" style="width:40px; height:40px; border-radius:var(--cover-radius); overflow:hidden; background:var(--panel); display:flex; align-items:center; justify-content:center;">${p.track.cover ? `<img src="${p.track.cover}" style="width:100%; height:100%; object-fit:cover;">` : "♪"}</div>
                  <div class="track-meta" style="display:flex; flex-direction:column;">
                    <span class="track-name" style="font-weight:700; color:var(--text);">${p.track.title}</span>
                    <span class="track-artists" style="font-size:12px; color:var(--dim);">${p.track.artists ? p.track.artists.map(a => a.name).join(", ") : "—"}</span>
                  </div>
                </div>
                <div class="track-actions">
                  <button class="btn-icon btn-play-row" data-id="${p.track.id}" style="background:none; border:none; color:var(--accent); cursor:pointer; font-size:16px;">▶</button>
                </div>
              </div>
            `
                    )
                    .join("")
            }
          </div>
        </div>
      </div>
    `;

    container.querySelectorAll(".pick-card").forEach((c) => {
      c.onclick = () => navigateTo(c.dataset.action);
    });
    container.querySelectorAll(".btn-play-row").forEach((btn) => {
      btn.onclick = (e) => {
        e.stopPropagation();
        const tr = state.history.find((p) => p.track.id === btn.dataset.id);
        if (tr) playTrack(tr.track);
      };
    });
    container.querySelectorAll(".track-row").forEach((row) => {
      row.onclick = () => {
        const tr = state.history.find((p) => p.track.id === row.dataset.id);
        if (tr) playTrack(tr.track);
      };
    });
  }

  function renderLikedScreen(container) {
    container.innerHTML = `
      <div class="screen-content">
        <div class="playlist-header" style="display:flex; align-items:center; gap:20px; margin-bottom:24px;">
          <div class="playlist-cover-large" style="width:120px; height:120px; border-radius:var(--cover-radius); background:rgba(244,167,187,0.2); display:flex; align-items:center; justify-content:center; font-size:44px; color:var(--accent);">♥</div>
          <div class="playlist-info">
            <span class="playlist-type" style="font-size:12px; font-weight:700; color:var(--accent);">ПЛЕЙЛИСТ</span>
            <h1 class="playlist-name" style="font-size:28px; font-weight:800; margin:4px 0; color:var(--text);">Любимые треки</h1>
            <span class="playlist-meta" style="font-size:13px; color:var(--dim);">${state.likedTracks.length} треков</span>
            <div class="playlist-buttons" style="margin-top:12px;">
              <button id="btn-play-liked-all" class="btn-primary" style="padding:10px 22px; border-radius:var(--chip-radius); background:var(--accent); color:var(--on-accent); border:none; font-weight:700; cursor:pointer;">Слушать всё</button>
            </div>
          </div>
        </div>

        <div class="tracks-table">
          <div class="table-header" style="display:flex; padding:8px 12px; font-size:12px; font-weight:700; color:var(--mute); border-bottom:1px solid var(--panel-border);">
            <span style="width:40px;">#</span>
            <span style="flex:1;">НАЗВАНИЕ</span>
            <span style="width:180px;">АЛЬБОМ</span>
            <span style="width:80px; text-align:right;">ВРЕМЯ</span>
          </div>
          <div id="liked-rows-container" style="display:flex; flex-direction:column; gap:4px; margin-top:6px;">
            ${
              state.likedTracks.length === 0
                ? '<div class="empty-notice" style="padding:32px 0; text-align:center;">В любимых пока нет треков. Нажмите на сердечко во время воспроизведения!</div>'
                : state.likedTracks
                    .map(
                      (t, idx) => `
                <div class="table-row ${state.currentTrack && state.currentTrack.id === t.id ? "playing" : ""}" data-id="${t.id}" data-idx="${idx}" style="display:flex; align-items:center; padding:8px 12px; border-radius:var(--cover-radius); cursor:pointer;">
                  <span style="width:40px; color:var(--mute); font-size:13px;">${idx + 1}</span>
                  <div style="flex:1; display:flex; align-items:center; gap:12px; min-width:0;">
                    <div style="width:36px; height:36px; border-radius:var(--cover-radius); overflow:hidden; background:var(--panel); display:flex; align-items:center; justify-content:center; flex-shrink:0;">${t.cover ? `<img src="${t.cover}" style="width:100%; height:100%; object-fit:cover;">` : "♪"}</div>
                    <div style="display:flex; flex-direction:column; overflow:hidden;">
                      <span style="font-weight:600; color:var(--text); white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">${t.title}</span>
                      <span style="font-size:12px; color:var(--dim); white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">${t.artists ? t.artists.map(a => a.name).join(", ") : "—"}</span>
                    </div>
                  </div>
                  <span style="width:180px; font-size:12.5px; color:var(--mute); white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">${t.album ? t.album.title : "—"}</span>
                  <span style="width:80px; text-align:right; font-family:monospace; font-size:12px; color:var(--dim);">${formatTime(t.durationSec)}</span>
                </div>
              `
                    )
                    .join("")
            }
          </div>
        </div>
      </div>
    `;

    const playAll = container.querySelector("#btn-play-liked-all");
    if (playAll) {
      playAll.onclick = () => {
        if (state.likedTracks.length > 0) {
          playTracks(state.likedTracks, 0);
        }
      };
    }

    container.querySelectorAll(".table-row").forEach((r) => {
      r.onclick = () => {
        const idx = parseInt(r.dataset.idx, 10);
        playTracks(state.likedTracks, idx);
      };
    });
  }

  function renderSearchScreen(container) {
    container.innerHTML = `
      <div class="screen-content">
        <h2 style="font-size:20px; font-weight:800; margin-bottom:16px;">Поиск по источникам</h2>
        <div id="search-results-area">
          <div class="empty-notice">Введите поисковый запрос в верхнюю панель или нажмите Enter.</div>
        </div>
      </div>
    `;
  }

  const ALL_STICKERS = [
    { id: "aga", code: "aga" },
    { id: "cooked", code: "coocked" },
    { id: "deal", code: "deal" },
    { id: "dumbass", code: "dumbass" },
    { id: "hmm", code: "hmm" },
    { id: "keepgambling", code: "keepgambling" },
    { id: "ragebait", code: "ragebait" },
    { id: "who", code: "who" },
    { id: "fans", code: "fans" },
    { id: "happy_guy", code: "happy_guy" },
    { id: "clock", code: "clock" },
    { id: "sad_smile", code: "sad_smile" },
    { id: "woman", code: "woman" },
    { id: "angry_man", code: "angry_man" },
    { id: "pink_cry", code: "pink_cry" },
    { id: "social_credit", code: "social_credit" },
    { id: "bald_man", code: "bald_man" },
    { id: "shocked_man", code: "shocked_man" },
    { id: "hairy_man", code: "hairy_man" },
    { id: "crying_man", code: "crying_man" },
    { id: "writing_man", code: "writing_man" },
    { id: "jaw_man", code: "jaw_man" },
    { id: "denied", code: "denied" },
    { id: "binoculars", code: "binoculars" },
    { id: "patrick", code: "patrick" },
    { id: "pepper", code: "pepper" },
    { id: "kitty_please", code: "kitty_please" },
    { id: "pig", code: "pig" },
    { id: "download_dog", code: "download_dog" },
    { id: "sweet_bird", code: "sweet_bird" },
    { id: "super_man", code: "super_man" },
    { id: "dumb_man", code: "dumb_man" },
    { id: "donkey", code: "donkey" },
    { id: "smart_man", code: "smart_man" },
    { id: "bunny", code: "bunny" },
    { id: "its_me_down", code: "its_me_down" },
    { id: "me", code: "me" }
  ];

  async function renderDownloadsScreen(container) {
    let downloads = [];
    try {
      downloads = (await invoke("get_downloads")) || [];
    } catch (e) {
      console.warn("get_downloads error:", e);
    }

    const hasCurrent = !!state.currentTrack;
    container.innerHTML = `
      <div class="screen-content" style="padding-bottom: 30px;">
        <div class="playlist-header" style="display:flex; align-items:center; justify-content:space-between; gap:20px; margin-bottom:24px;">
          <div style="display:flex; align-items:center; gap:20px;">
            <div class="playlist-cover-large" style="width:110px; height:110px; border-radius:var(--cover-radius); background:var(--fill); display:flex; align-items:center; justify-content:center; font-size:40px; color:var(--accent);">⬇</div>
            <div class="playlist-info">
              <span class="playlist-type" style="font-size:12px; font-weight:700; color:var(--accent); text-transform:uppercase;">Офлайн медиатека</span>
              <h1 class="playlist-name" style="font-size:28px; font-weight:800; margin:4px 0; color:var(--text);">Загруженные треки & клипы</h1>
              <span class="playlist-meta" style="font-size:13px; color:var(--dim);">Всего файлов: ${downloads.length} • Папка: %USERPROFILE%\\Music\\Drxwnify</span>
            </div>
          </div>
          <div class="downloads-actions" style="display:flex; gap:10px;">
            <button id="btn-dl-curr-track" class="chip ${hasCurrent ? 'chip-active' : ''}" style="cursor:${hasCurrent ? 'pointer' : 'not-allowed'}; opacity:${hasCurrent ? 1 : 0.5}; font-weight:700; display:flex; align-items:center; gap:6px;">
              <span>⬇</span> Скачать трек (MP3)
            </button>
            <button id="btn-dl-curr-clip" class="chip ${hasCurrent ? 'chip-active' : ''}" style="cursor:${hasCurrent ? 'pointer' : 'not-allowed'}; opacity:${hasCurrent ? 1 : 0.5}; font-weight:700; display:flex; align-items:center; gap:6px;">
              <span>🎬</span> Скачать клип (MP4)
            </button>
          </div>
        </div>

        <div class="downloads-list" style="display:flex; flex-direction:column; gap:8px;">
          ${
            downloads.length === 0
              ? '<div class="empty-notice" style="padding:40px 0; text-align:center; color:var(--dim);">Здесь пока пусто. Вы можете скачать любой трек или видеоклип через плеер или поиск.</div>'
              : downloads
                  .map(
                    (d) => `
              <div class="download-item-row" data-id="${d.id}" style="display:flex; align-items:center; justify-content:space-between; padding:12px 16px; background:var(--panel); border:1px solid var(--panel-border); border-radius:var(--cover-radius);">
                <div style="display:flex; align-items:center; gap:14px; flex:1; min-width:0;">
                  <div style="width:42px; height:42px; border-radius:8px; background:var(--fill); display:flex; align-items:center; justify-content:center; font-size:20px; color:var(--accent); flex-shrink:0;">
                    ${d.kind === 'clip' ? '🎬' : '♫'}
                  </div>
                  <div style="display:flex; flex-direction:column; min-width:0; overflow:hidden;">
                    <span style="font-weight:700; font-size:14px; color:var(--text); white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">${d.title}</span>
                    <span style="font-size:12px; color:var(--dim); white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">${d.artist} • <span style="font-family:monospace; color:var(--accent);">${d.quality}</span> • ${new Date(d.downloadedAt).toLocaleString()}</span>
                  </div>
                </div>
                <div style="display:flex; align-items:center; gap:10px; margin-left:16px;">
                  ${d.kind !== 'clip' ? `<button class="btn-play-dl chip" data-id="${d.id}" style="cursor:pointer; font-weight:700;">▶ Воспроизвести</button>` : ''}
                  <button class="btn-delete-dl chip" data-id="${d.id}" style="cursor:pointer; color:#f87171;">✕ Удалить</button>
                </div>
              </div>
            `
                  )
                  .join("")
          }
        </div>
      </div>
    `;

    const dlTrackBtn = container.querySelector("#btn-dl-curr-track");
    if (dlTrackBtn && hasCurrent) {
      dlTrackBtn.onclick = async () => {
        dlTrackBtn.textContent = "⏳ Скачивание...";
        try {
          await invoke("download_track", { track: state.currentTrack, streamUrl: null });
          renderDownloadsScreen(container);
        } catch (e) {
          alert("Ошибка скачивания трека: " + e);
          dlTrackBtn.textContent = "⬇ Скачать трек (MP3)";
        }
      };
    }

    const dlClipBtn = container.querySelector("#btn-dl-curr-clip");
    if (dlClipBtn && hasCurrent) {
      dlClipBtn.onclick = async () => {
        dlClipBtn.textContent = "⏳ Скачивание клипа...";
        try {
          const artistName = state.currentTrack.artists && state.currentTrack.artists[0] ? state.currentTrack.artists[0].name : "Drxwnify";
          await invoke("download_clip", { id: state.currentTrack.id, title: state.currentTrack.title, artist: artistName, videoUrl: null });
          renderDownloadsScreen(container);
        } catch (e) {
          alert("Ошибка скачивания клипа: " + e);
          dlClipBtn.textContent = "🎬 Скачать клип (MP4)";
        }
      };
    }

    container.querySelectorAll(".btn-delete-dl").forEach((btn) => {
      btn.onclick = async () => {
        await invoke("remove_download", { id: btn.dataset.id });
        renderDownloadsScreen(container);
      };
    });

    container.querySelectorAll(".btn-play-dl").forEach((btn) => {
      btn.onclick = () => {
        const item = downloads.find((d) => d.id === btn.dataset.id);
        if (item) {
          playTrack({
            id: item.id,
            title: item.title,
            artists: [{ id: "artist", name: item.artist }],
            album: null,
            durationSec: 180,
            cover: null,
          });
        }
      };
    });
  }

  function renderHistoryScreen(container) {
    container.innerHTML = `
      <div class="screen-content">
        <div class="history-top" style="display:flex; justify-content:space-between; align-items:center; margin-bottom:20px;">
          <h1 style="font-size:24px; font-weight:800; color:var(--text);">История прослушиваний</h1>
          <button id="btn-clear-history" class="chip" style="cursor:pointer;">Очистить историю</button>
        </div>
        <div class="history-list" style="display:flex; flex-direction:column; gap:6px;">
          ${
            state.history.length === 0
              ? '<div class="empty-notice">История пуста.</div>'
              : state.history
                  .map(
                    (p) => `
              <div class="track-row" data-id="${p.track.id}" style="display:flex; align-items:center; justify-content:space-between; padding:8px 12px; background:var(--fill); border-radius:var(--cover-radius); cursor:pointer;">
                <div class="track-left" style="display:flex; align-items:center; gap:12px;">
                  <div class="track-thumb" style="width:40px; height:40px; border-radius:var(--cover-radius); overflow:hidden; background:var(--panel); display:flex; align-items:center; justify-content:center;">${p.track.cover ? `<img src="${p.track.cover}" style="width:100%; height:100%; object-fit:cover;">` : "♪"}</div>
                  <div class="track-meta" style="display:flex; flex-direction:column;">
                    <span class="track-name" style="font-weight:700; color:var(--text);">${p.track.title}</span>
                    <span class="track-artists" style="font-size:12px; color:var(--dim);">${p.track.artists ? p.track.artists.map(a => a.name).join(", ") : "—"}</span>
                  </div>
                </div>
                <div class="track-time" style="font-size:12px; color:var(--mute);">${new Date(p.timestamp).toLocaleTimeString()}</div>
              </div>
            `
                  )
                  .join("")
          }
        </div>
      </div>
    `;

    const clearBtn = container.querySelector("#btn-clear-history");
    if (clearBtn) {
      clearBtn.onclick = async () => {
        await invoke("clear_history");
        state.history = [];
        renderHistoryScreen(container);
      };
    }

    container.querySelectorAll(".track-row").forEach((row) => {
      row.onclick = () => {
        const item = state.history.find((p) => p.track.id === row.dataset.id);
        if (item) playTrack(item.track);
      };
    });
  }

  async function renderFriendsScreen(container) {
    let friendList = [];
    try {
      friendList = (await invoke("get_friends")) || [];
    } catch (e) {
      console.warn("get_friends error:", e);
    }
    if (!friendList || friendList.length === 0) {
      friendList = [
        { pubkey: "npub1slvx666", name: "slvx666 (drxwnslvt)", isOnline: true, currentTrack: { title: "Drown", artists: [{ name: "Bring Me The Horizon" }] } },
        { pubkey: "npub1fend1x", name: "fend1x", isOnline: true, currentTrack: { title: "Resonance", artists: [{ name: "HOME" }] } },
        { pubkey: "npub1clavdiy", name: "clavdiy", isOnline: false, currentTrack: null },
      ];
    }

    const activeFriend = state.activeChatFriend || friendList[0];
    let messages = [];
    try {
      messages = (await invoke("get_messages", { friendPubkey: activeFriend.pubkey })) || [];
    } catch (e) {
      console.warn("get_messages error:", e);
    }

    if (messages.length === 0) {
      messages = [
        {
          id: "m1",
          fromPubkey: activeFriend.pubkey,
          toPubkey: "me",
          text: "Привет! Слушаешь новый релиз?",
          timestamp: Date.now() - 1000 * 60 * 15,
          trackAttachment: null,
        },
        {
          id: "m2",
          fromPubkey: "me",
          toPubkey: activeFriend.pubkey,
          text: "Да, слушаю через Drxwnify Full на Rust! Звучит шикарно.",
          timestamp: Date.now() - 1000 * 60 * 12,
          trackAttachment: null,
        },
        {
          id: "m3",
          fromPubkey: activeFriend.pubkey,
          toPubkey: "me",
          text: "[sticker:aga]",
          timestamp: Date.now() - 1000 * 60 * 5,
          trackAttachment: null,
        },
      ];
    }

    function renderMessageBody(msg) {
      let html = "";
      if (msg.text) {
        if (msg.text.startsWith("[sticker:") && msg.text.endsWith("]")) {
          const stickerId = msg.text.slice(9, -1);
          html += `<img src="stickers/${stickerId}.webp" alt="${stickerId}" class="chat-sticker" style="width:110px; height:110px; object-fit:contain; display:block; filter:drop-shadow(0 2px 8px rgba(0,0,0,0.2));" />`;
        } else {
          html += `<div style="font-size:13.5px; word-break:break-word;">${msg.text}</div>`;
        }
      }
      if (msg.trackAttachment || msg.track_attachment) {
        const t = msg.trackAttachment || msg.track_attachment;
        const artName = t.artists && t.artists[0] ? t.artists[0].name : "Drxwnify";
        html += `
          <div class="chat-track-card" data-id="${t.id}" style="margin-top:8px; padding:8px 10px; background:rgba(0,0,0,0.25); border-radius:8px; display:flex; align-items:center; gap:10px; cursor:pointer;">
            <div style="font-size:20px; color:var(--accent);">♫</div>
            <div style="flex:1; min-width:0;">
              <div style="font-size:12.5px; font-weight:700; white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">${t.title}</div>
              <div style="font-size:11px; opacity:0.8;">${artName}</div>
            </div>
            <button class="btn-play-chat-track" style="border:none; background:var(--accent); color:var(--on-accent); border-radius:50%; width:28px; height:28px; display:flex; align-items:center; justify-content:center; cursor:pointer; font-size:11px;">▶</button>
          </div>
        `;
      }
      return html;
    }

    container.innerHTML = `
      <div class="chathub-container" style="display:flex; height:600px; gap:16px;">
        <div class="chathub-sidebar" style="width:260px; background:var(--panel); border:1px solid var(--panel-border); border-radius:var(--panel-radius); padding:16px; display:flex; flex-direction:column;">
          <div class="chathub-header" style="display:flex; justify-content:space-between; align-items:center; margin-bottom:14px;">
            <h3 style="font-size:15px; font-weight:800; color:var(--text);">Друзья & Nostr</h3>
            <span class="badge" style="font-size:10px; background:var(--accent); color:var(--on-accent); padding:2px 8px; border-radius:12px;">Relays: 5 ok</span>
          </div>
          <div class="chathub-friends-list" style="display:flex; flex-direction:column; gap:6px; overflow-y:auto;">
            ${friendList
              .map(
                (f) => `
              <div class="chathub-friend-item chip ${activeFriend.pubkey === f.pubkey ? "active" : ""}" data-pubkey="${f.pubkey}" style="display:flex; align-items:center; gap:10px; padding:8px 10px; cursor:pointer;">
                <div style="width:32px; height:32px; border-radius:50%; background:var(--accent); color:var(--on-accent); display:flex; align-items:center; justify-content:center; font-weight:800;">${f.name.charAt(0).toUpperCase()}</div>
                <div style="display:flex; flex-direction:column; min-width:0; flex:1;">
                  <span style="font-weight:700; font-size:13px; color:var(--text); white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">${f.name}</span>
                  <span style="font-size:11px; color:var(--dim); white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">${f.currentTrack ? `♫ ${f.currentTrack.title}` : (f.isOnline ? "В сети" : "Не в сети")}</span>
                </div>
              </div>
            `
              )
              .join("")}
          </div>
        </div>

        <div class="chathub-chat-pane" style="flex:1; background:var(--panel); border:1px solid var(--panel-border); border-radius:var(--panel-radius); padding:16px; display:flex; flex-direction:column; position:relative;">
          <div class="chat-header" style="border-bottom:1px solid var(--panel-border); padding-bottom:10px; margin-bottom:12px; display:flex; justify-content:space-between; align-items:center;">
            <div style="display:flex; align-items:center; gap:8px;">
              <span style="font-size:16px; font-weight:800; color:var(--text);">${activeFriend.name}</span>
              ${activeFriend.isOnline ? '<span style="width:8px; height:8px; border-radius:50%; background:#22c55e;"></span>' : ''}
            </div>
            <span style="font-size:11.5px; color:var(--mute);">Nostr Encrypted (Kind 4)</span>
          </div>

          <div class="chat-messages" id="chat-messages-container" style="flex:1; overflow-y:auto; display:flex; flex-direction:column; gap:10px; padding-right:6px;">
            ${messages
              .map((m) => {
                const isMe = m.fromPubkey === "me" || m.from_pubkey === "me";
                return `
                <div style="align-self:${isMe ? 'flex-end' : 'flex-start'}; max-width:70%; background:${isMe ? 'var(--accent)' : 'var(--fill)'}; color:${isMe ? 'var(--on-accent)' : 'var(--text)'}; padding:10px 14px; border-radius:var(--cover-radius); box-shadow:0 1px 4px rgba(0,0,0,0.1);">
                  ${renderMessageBody(m)}
                  <div style="font-size:10px; opacity:0.8; margin-top:4px; text-align:right;">${new Date(m.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</div>
                </div>
              `;
              })
              .join("")}
          </div>

          <!-- Sticker Picker Popover -->
          <div id="chat-sticker-picker" class="hidden" style="position:absolute; bottom:64px; left:16px; right:16px; height:240px; background:var(--panel); border:1px solid var(--panel-border); border-radius:var(--panel-radius); padding:12px; box-shadow:0 8px 30px rgba(0,0,0,0.4); z-index:100; display:none; flex-direction:column;">
            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:10px;">
              <span style="font-size:13px; font-weight:800; color:var(--text);">Стикеры (${ALL_STICKERS.length})</span>
              <button id="btn-close-stickers" style="background:none; border:none; color:var(--dim); font-size:16px; cursor:pointer;">✕</button>
            </div>
            <div class="stickers-grid" style="flex:1; overflow-y:auto; display:grid; grid-template-columns:repeat(auto-fill, minmax(64px, 1fr)); gap:8px;">
              ${ALL_STICKERS.map(s => `
                <div class="sticker-btn-item" data-id="${s.id}" title="${s.code}" style="cursor:pointer; display:flex; align-items:center; justify-content:center; padding:6px; border-radius:8px; background:var(--fill); transition:transform 0.15s ease;">
                  <img src="stickers/${s.id}.webp" alt="${s.id}" style="width:52px; height:52px; object-fit:contain;" />
                </div>
              `).join("")}
            </div>
          </div>

          <div class="chat-input-bar" style="display:flex; gap:8px; margin-top:12px; align-items:center;">
            <button id="btn-toggle-stickers" class="btn-icon" title="Стикеры" style="width:40px; height:40px; border-radius:var(--chip-radius); background:var(--fill); border:1px solid var(--panel-border); cursor:pointer; font-size:18px; display:flex; align-items:center; justify-content:center;">
              🏷️
            </button>
            <button id="btn-attach-track" class="btn-icon" title="Прикрепить текущий трек" style="width:40px; height:40px; border-radius:var(--chip-radius); background:var(--fill); border:1px solid var(--panel-border); cursor:pointer; font-size:18px; display:flex; align-items:center; justify-content:center;">
              ♫
            </button>
            <input type="text" id="chat-text-input" placeholder="Написать сообщение через Nostr..." style="flex:1; padding:10px 14px; background:var(--fill); border:1px solid var(--panel-border); border-radius:var(--chip-radius); color:var(--text); font-family:var(--font-main);">
            <button id="btn-chat-send" class="btn-primary" style="padding:8px 18px; height:40px; border-radius:var(--chip-radius); background:var(--accent); color:var(--on-accent); border:none; font-weight:700; cursor:pointer;">Отправить</button>
          </div>
        </div>
      </div>
    `;

    const msgContainer = container.querySelector("#chat-messages-container");
    if (msgContainer) msgContainer.scrollTop = msgContainer.scrollHeight;

    container.querySelectorAll(".chathub-friend-item").forEach((it) => {
      it.onclick = () => {
        state.activeChatFriend = friendList.find((f) => f.pubkey === it.dataset.pubkey);
        renderFriendsScreen(container);
      };
    });

    const stickerPicker = container.querySelector("#chat-sticker-picker");
    const toggleStickersBtn = container.querySelector("#btn-toggle-stickers");
    const closeStickersBtn = container.querySelector("#btn-close-stickers");

    if (toggleStickersBtn && stickerPicker) {
      toggleStickersBtn.onclick = () => {
        const isHidden = stickerPicker.style.display === "none" || !stickerPicker.style.display;
        stickerPicker.style.display = isHidden ? "flex" : "none";
      };
    }

    if (closeStickersBtn && stickerPicker) {
      closeStickersBtn.onclick = () => {
        stickerPicker.style.display = "none";
      };
    }

    container.querySelectorAll(".sticker-btn-item").forEach((sBtn) => {
      sBtn.onclick = async () => {
        const stickerId = sBtn.dataset.id;
        stickerPicker.style.display = "none";
        try {
          await invoke("send_message", {
            toPubkey: activeFriend.pubkey,
            text: `[sticker:${stickerId}]`,
            track: null,
          });
          renderFriendsScreen(container);
        } catch (e) {
          console.error("Failed to send sticker:", e);
        }
      };
    });

    const attachTrackBtn = container.querySelector("#btn-attach-track");
    if (attachTrackBtn) {
      attachTrackBtn.onclick = async () => {
        if (!state.currentTrack) {
          alert("Сейчас ничего не воспроизводится для прикрепления.");
          return;
        }
        try {
          await invoke("send_message", {
            toPubkey: activeFriend.pubkey,
            text: `Делюсь треком: ${state.currentTrack.title}`,
            track: state.currentTrack,
          });
          renderFriendsScreen(container);
        } catch (e) {
          console.error("Failed to attach track:", e);
        }
      };
    }

    const sendBtn = container.querySelector("#btn-chat-send");
    const input = container.querySelector("#chat-text-input");
    if (sendBtn && input) {
      sendBtn.onclick = async () => {
        const text = input.value.trim();
        if (!text) return;
        input.value = "";
        try {
          await invoke("send_message", {
            toPubkey: activeFriend.pubkey,
            text,
            track: null,
          });
          renderFriendsScreen(container);
        } catch (e) {
          console.error("Failed to send message:", e);
        }
      };
      input.onkeydown = (e) => {
        if (e.key === "Enter") sendBtn.click();
      };
    }

    container.querySelectorAll(".btn-play-chat-track").forEach((btn) => {
      btn.onclick = (e) => {
        e.stopPropagation();
        const card = btn.closest(".chat-track-card");
        if (card && activeFriend.currentTrack) {
          playTrack(activeFriend.currentTrack);
        }
      };
    });
  }

  function renderReviewsScreen(container) {
    container.innerHTML = `
      <div class="screen-content">
        <h1 style="font-size:24px; font-weight:800; color:var(--text); margin-bottom:4px;">Обзоры drxwnslvt</h1>
        <p style="font-size:13.5px; color:var(--dim); margin-bottom:20px;">Авторские рецензии и подборки от создателя Drxwnify.</p>
        <div class="reviews-grid" style="display:grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap:16px;">
          <div class="review-card" style="background:var(--panel); border:1px solid var(--panel-border); border-radius:var(--panel-radius); padding:18px;">
            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:8px;">
              <span style="font-size:16px; font-weight:800; color:var(--text);">Sempiternal</span>
              <span style="font-size:12px; font-weight:700; color:var(--accent);">★ 9.8 / 10</span>
            </div>
            <p style="font-size:13px; color:var(--dim); line-height:1.5;">Монументальный релиз, определивший звучание альтернативной сцены на целое десятилетие.</p>
          </div>
          <div class="review-card" style="background:var(--panel); border:1px solid var(--panel-border); border-radius:var(--panel-radius); padding:18px;">
            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:8px;">
              <span style="font-size:16px; font-weight:800; color:var(--text);">Holy Hell</span>
              <span style="font-size:12px; font-weight:700; color:var(--accent);">★ 9.5 / 10</span>
            </div>
            <p style="font-size:13px; color:var(--dim); line-height:1.5;">Искренняя эмоциональная ярость и потрясающая звуковая глубина оркестровых аранжировок.</p>
          </div>
        </div>
      </div>
    `;
  }

  function renderSettingsScreen(container) {
    container.innerHTML = `
      <div class="screen-content">
        <div class="section-block" style="margin-bottom:24px;">
          <h1 style="font-size:26px; font-weight:800; color:var(--text); margin-bottom:6px;">Настройки</h1>
          <p style="font-size:13.5px; color:var(--dim);">Параметры тем, аудиосистемы и интеграций.</p>
        </div>

        <div class="section-block" style="margin-bottom:24px;">
          <div class="section-title" style="font-size:14px; font-weight:700; text-transform:uppercase; letter-spacing:1px; color:var(--mute); margin-bottom:12px;">Стиль интерфейса (Look)</div>
          <div style="display:flex; flex-wrap:wrap; gap:8px; margin-bottom:16px;">
            <button class="chip btn-look-opt ${state.look === 'glass' ? 'active' : ''}" data-look="glass">Glass</button>
            <button class="chip btn-look-opt ${state.look === 'terminal' ? 'active' : ''}" data-look="terminal">Terminal</button>
            <button class="chip btn-look-opt ${state.look === 'neon' ? 'active' : ''}" data-look="neon">Neon</button>
            <button class="chip btn-look-opt ${state.look === 'island' ? 'active' : ''}" data-look="island">Island</button>
            <button class="chip btn-look-opt ${state.look === 'blossom' ? 'active' : ''}" data-look="blossom">Blossom</button>
            <button class="chip btn-look-opt ${state.look === 'quilt' ? 'active' : ''}" data-look="quilt">Quilt</button>
            <button class="chip btn-look-opt ${state.look === 'atlas' ? 'active' : ''}" data-look="atlas">Atlas</button>
            <button class="chip btn-look-opt ${state.look === 'rite' ? 'active' : ''}" data-look="rite">Rite</button>
          </div>

          <div class="section-title" style="font-size:14px; font-weight:700; text-transform:uppercase; letter-spacing:1px; color:var(--mute); margin-bottom:12px;">Цветовая тема</div>
          <div class="theme-options-grid" style="display:grid; grid-template-columns: repeat(3, 1fr); gap:8px; margin-bottom:14px;">
            <div class="theme-card ${state.theme === 'tategaki' ? 'active' : ''}" data-theme="tategaki">Татэгаки (Tategaki)</div>
            <div class="theme-card ${state.theme === 'gruvbox' ? 'active' : ''}" data-theme="gruvbox">Gruvbox</div>
            <div class="theme-card ${state.theme === 'retrowave' ? 'active' : ''}" data-theme="retrowave">Retrowave</div>
            <div class="theme-card ${state.theme === 'depth' ? 'active' : ''}" data-theme="depth">Глубина (Depth)</div>
            <div class="theme-card ${state.theme === 'sakura' ? 'active' : ''}" data-theme="sakura">Сакура (Sakura)</div>
            <div class="theme-card ${state.theme === 'cozy' ? 'active' : ''}" data-theme="cozy">Cozy</div>
            <div class="theme-card ${state.theme === 'observatory' ? 'active' : ''}" data-theme="observatory">Обсерватория</div>
            <div class="theme-card ${state.theme === 'satanic' ? 'active' : ''}" data-theme="satanic">Satanic</div>
            <div class="theme-card ${state.theme === 'mono' ? 'active' : ''}" data-theme="mono">Монохром (Mono)</div>
          </div>

          <div class="theme-mode-row" style="display:flex; gap:16px; font-size:13.5px; color:var(--text); margin-bottom:20px;">
            <label style="cursor:pointer;"><input type="radio" name="settings-theme-mode" value="night" ${state.mode === 'night' ? 'checked' : ''}> Ночная палитра (Night)</label>
            <label style="cursor:pointer;"><input type="radio" name="settings-theme-mode" value="day" ${state.mode === 'day' ? 'checked' : ''}> Дневная палитра (Day)</label>
          </div>
        </div>

        <div class="section-block" style="margin-bottom:24px;">
          <div class="section-title" style="font-size:14px; font-weight:700; text-transform:uppercase; letter-spacing:1px; color:var(--mute); margin-bottom:12px;">Учетные записи & Источники</div>
          <div style="display:flex; flex-direction:column; gap:12px; max-width:540px;">
            <div>
              <label style="font-size:12px; color:var(--dim); display:block; margin-bottom:4px;">Токен VK Music:</label>
              <input type="text" id="input-vk-token" placeholder="vk1.a.xxxx..." style="width:100%; box-sizing:border-box; padding:8px 12px; background:var(--fill); border:1px solid var(--panel-border); border-radius:var(--cover-radius); color:var(--text); font-family:monospace;">
            </div>
            <button id="btn-save-accounts" class="btn-primary" style="align-self:flex-start; padding:8px 18px; border-radius:var(--chip-radius); background:var(--accent); color:var(--on-accent); border:none; font-weight:700; cursor:pointer;">Сохранить аккаунты</button>
            <div id="accounts-save-status" style="font-size:12px; color:var(--accent);"></div>
          </div>
        </div>

        <div class="section-block">
          <div class="section-title" style="font-size:14px; font-weight:700; text-transform:uppercase; letter-spacing:1px; color:var(--mute); margin-bottom:8px;">О программе</div>
          <p style="font-size:13px; color:var(--dim); line-height:1.6;">
            Drxwnify Full Edition — 100% Rust Core (Wasapi, Symphonia, Tokio, InnerTube, VK Music, SoundCloud, LRCLIB).<br>
            Dual Edition архитектура. Версия 0.4.0.
          </p>
        </div>
      </div>
    `;

    // Look buttons
    container.querySelectorAll(".btn-look-opt").forEach((btn) => {
      btn.onclick = () => {
        const look = btn.dataset.look;
        applyTheme(state.theme, state.mode, look);
        renderSettingsScreen(container);
      };
    });

    // Theme cards
    container.querySelectorAll(".theme-card").forEach((card) => {
      card.onclick = () => {
        const theme = card.dataset.theme;
        applyTheme(theme, state.mode, state.look);
        renderSettingsScreen(container);
      };
    });

    // Day/Night radio
    container.querySelectorAll('input[name="settings-theme-mode"]').forEach((r) => {
      r.onchange = (e) => {
        applyTheme(state.theme, e.target.value, state.look);
        renderSettingsScreen(container);
      };
    });

    // Accounts save
    const btnSaveAcc = container.querySelector("#btn-save-accounts");
    const vkInput = container.querySelector("#input-vk-token");
    const saveStatus = container.querySelector("#accounts-save-status");
    if (btnSaveAcc && vkInput) {
      btnSaveAcc.onclick = async () => {
        try {
          const acc = await invoke("get_accounts");
          if (acc) {
            acc.vkToken = vkInput.value.trim() || null;
            await invoke("save_accounts", { accounts: acc });
            if (saveStatus) saveStatus.textContent = "Успешно сохранено!";
          }
        } catch (err) {
          if (saveStatus) saveStatus.textContent = `Ошибка: ${err}`;
        }
      };
    }
  }

  // --- Search Input Trigger ---
  if (els.searchInput) {
    let debounceTimer = null;
    els.searchInput.addEventListener("input", (e) => {
      const q = e.target.value.trim();
      if (els.searchClear) {
        if (q) els.searchClear.classList.remove("hidden");
        else els.searchClear.classList.add("hidden");
      }
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => {
        if (q) performSearch(q);
      }, 300);
    });

    els.searchInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        clearTimeout(debounceTimer);
        const q = els.searchInput.value.trim();
        if (q) performSearch(q);
      }
    });

    if (els.searchClear) {
      els.searchClear.onclick = () => {
        els.searchInput.value = "";
        els.searchClear.classList.add("hidden");
      };
    }
  }

  async function performSearch(query) {
    navigateTo("search");
    const container = document.getElementById("search-results-area");
    if (!container) return;
    container.innerHTML = '<div class="loading-spinner" style="padding:24px 0; color:var(--dim);">Поиск по источникам (YouTube Music, VK, SoundCloud)...</div>';

    try {
      const res = await invoke("search", { query });
      if (!res || !res.tracks || res.tracks.length === 0) {
        container.innerHTML = `<div class="empty-notice">По запросу "${query}" ничего не найдено.</div>`;
        return;
      }

      container.innerHTML = `
        <div class="search-tracks-list" style="display:flex; flex-direction:column; gap:6px;">
          ${res.tracks
            .map(
              (t, idx) => `
            <div class="track-row" data-id="${t.id}" data-idx="${idx}" style="display:flex; align-items:center; justify-content:space-between; padding:8px 12px; background:var(--fill); border-radius:var(--cover-radius); cursor:pointer;">
              <div class="track-left" style="display:flex; align-items:center; gap:12px; flex:1; min-width:0;">
                <div class="track-thumb" style="width:42px; height:42px; border-radius:var(--cover-radius); overflow:hidden; background:var(--panel); display:flex; align-items:center; justify-content:center; flex-shrink:0;">${t.cover ? `<img src="${t.cover}" style="width:100%; height:100%; object-fit:cover;">` : "♪"}</div>
                <div class="track-meta" style="display:flex; flex-direction:column; min-width:0; overflow:hidden;">
                  <span class="track-name" style="font-weight:700; color:var(--text); white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">${t.title}</span>
                  <span class="track-artists" style="font-size:12px; color:var(--dim); white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">${t.artists ? t.artists.map(a => a.name).join(", ") : "—"}</span>
                </div>
              </div>
              <div class="track-time" style="font-size:12px; font-family:monospace; color:var(--dim); margin:0 16px;">${formatTime(t.durationSec)}</div>
              <div class="track-actions" style="display:flex; align-items:center;">
                <button class="btn-icon btn-dl-track-search" data-id="${t.id}" title="Скачать MP3" style="background:none; border:none; color:var(--dim); cursor:pointer; font-size:14px; margin-right:8px;">⬇</button>
                <button class="btn-icon btn-dl-clip-search" data-id="${t.id}" title="Скачать клип (MP4)" style="background:none; border:none; color:var(--dim); cursor:pointer; font-size:14px; margin-right:10px;">🎬</button>
                <button class="btn-icon btn-play-search" data-id="${t.id}" style="background:none; border:none; color:var(--accent); cursor:pointer; font-size:18px;">▶</button>
              </div>
            </div>
          `
            )
            .join("")}
        </div>
      `;

      container.querySelectorAll(".track-row").forEach((row) => {
        row.onclick = () => {
          const idx = parseInt(row.dataset.idx, 10);
          playTracks(res.tracks, idx);
        };
      });

      container.querySelectorAll(".btn-dl-track-search").forEach((b) => {
        b.onclick = async (e) => {
          e.stopPropagation();
          const t = res.tracks.find((item) => item.id === b.dataset.id);
          if (t) {
            b.textContent = "⏳";
            try {
              await invoke("download_track", { track: t, streamUrl: null });
              b.textContent = "✓";
            } catch (err) {
              console.error(err);
              b.textContent = "✕";
            }
          }
        };
      });

      container.querySelectorAll(".btn-dl-clip-search").forEach((b) => {
        b.onclick = async (e) => {
          e.stopPropagation();
          const t = res.tracks.find((item) => item.id === b.dataset.id);
          if (t) {
            b.textContent = "⏳";
            try {
              const artName = t.artists && t.artists[0] ? t.artists[0].name : "Drxwnify";
              await invoke("download_clip", { id: t.id, title: t.title, artist: artName, videoUrl: null });
              b.textContent = "✓";
            } catch (err) {
              console.error(err);
              b.textContent = "✕";
            }
          }
        };
      });

      container.querySelectorAll(".btn-play-search").forEach((b) => {
        b.onclick = (e) => {
          e.stopPropagation();
          const t = res.tracks.find((item) => item.id === b.dataset.id);
          if (t) playTrack(t);
        };
      });
    } catch (e) {
      container.innerHTML = `<div class="error-notice">Ошибка поиска: ${e}</div>`;
    }
  }

  // --- Player Core Invocations ---
  async function playTrack(track) {
    state.currentTrack = track;
    updateTrackMetadataUI(track);
    await invoke("play_track", { track });
    state.isPlaying = true;
    updatePlayPauseUI();
    fetchLyrics(track);
  }

  async function playTracks(tracks, startIndex) {
    if (!tracks || tracks.length === 0) return;
    state.queue = tracks;
    state.queueIndex = startIndex;
    const current = tracks[startIndex];
    state.currentTrack = current;
    updateTrackMetadataUI(current);
    await invoke("play_tracks", { tracks, startIndex });
    state.isPlaying = true;
    updatePlayPauseUI();
    fetchLyrics(current);
    renderQueueList();
  }

  function updateTrackMetadataUI(track) {
    if (!track) return;
    if (els.playerTitle) els.playerTitle.textContent = track.title;
    if (els.playerArtist) {
      const arts = track.artists ? track.artists.map((a) => a.name).join(", ") : "—";
      els.playerArtist.textContent = arts;
    }
    if (els.playerCover) {
      els.playerCover.src = track.cover || "";
    }
    if (els.polyTrack) {
      els.polyTrack.textContent = `:: ${track.title}`;
    }
    // Update like button state
    const isLiked = state.likedTracks.some((t) => t.id === track.id);
    if (els.playerBtnLike) {
      if (isLiked) els.playerBtnLike.classList.add("liked");
      else els.playerBtnLike.classList.remove("liked");
    }
  }

  function updatePlayPauseUI() {
    if (state.isPlaying) {
      if (els.iconPlay) els.iconPlay.classList.add("hidden");
      if (els.iconPause) els.iconPause.classList.remove("hidden");
    } else {
      if (els.iconPlay) els.iconPlay.classList.remove("hidden");
      if (els.iconPause) els.iconPause.classList.add("hidden");
    }
  }

  if (els.btnPlayPause) {
    els.btnPlayPause.onclick = async () => {
      state.isPlaying = !state.isPlaying;
      updatePlayPauseUI();
      await invoke("toggle_play_pause");
    };
  }

  if (els.btnPrev) {
    els.btnPrev.onclick = async () => {
      await invoke("previous_track");
    };
  }
  if (els.btnNext) {
    els.btnNext.onclick = async () => {
      await invoke("next_track");
    };
  }
  if (els.btnShuffle) {
    els.btnShuffle.onclick = async () => {
      state.isShuffled = !state.isShuffled;
      els.btnShuffle.classList.toggle("active", state.isShuffled);
      await invoke("shuffle");
    };
  }
  if (els.btnRepeat) {
    els.btnRepeat.onclick = async () => {
      if (state.repeatMode === "off") state.repeatMode = "all";
      else if (state.repeatMode === "all") state.repeatMode = "one";
      else state.repeatMode = "off";

      els.btnRepeat.classList.toggle("active", state.repeatMode !== "off");
      if (els.repeatBadge) {
        if (state.repeatMode === "one") els.repeatBadge.classList.remove("hidden");
        else els.repeatBadge.classList.add("hidden");
      }
      await invoke("set_repeat_mode", { mode: state.repeatMode });
    };
  }

  // --- Like Toggle ---
  if (els.playerBtnLike) {
    els.playerBtnLike.onclick = async () => {
      if (!state.currentTrack) return;
      const isNowLiked = await invoke("toggle_like", { track: state.currentTrack });
      if (isNowLiked) {
        state.likedTracks.unshift(state.currentTrack);
        els.playerBtnLike.classList.add("liked");
      } else {
        state.likedTracks = state.likedTracks.filter((t) => t.id !== state.currentTrack.id);
        els.playerBtnLike.classList.remove("liked");
      }
      if (els.likedCount) els.likedCount.textContent = state.likedTracks.length;
    };
  }

  // --- Seeking ---
  if (els.seekBarTrack) {
    let isSeeking = false;
    const seekHandler = async (e) => {
      const rect = els.seekBarTrack.getBoundingClientRect();
      const pos = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      const totalSec = state.currentTrack ? state.currentTrack.durationSec : 180;
      const targetMs = Math.floor(pos * totalSec * 1000);
      state.positionMs = targetMs;
      updateTimelineUI();
      await invoke("seek", { positionMs: targetMs });
    };

    els.seekBarTrack.addEventListener("mousedown", (e) => {
      isSeeking = true;
      seekHandler(e);
    });
    window.addEventListener("mousemove", (e) => {
      if (isSeeking) seekHandler(e);
    });
    window.addEventListener("mouseup", () => {
      isSeeking = false;
    });
  }

  // --- Volume & Mute ---
  if (els.volBarTrack) {
    let isVolChanging = false;
    const volHandler = async (e) => {
      const rect = els.volBarTrack.getBoundingClientRect();
      const val = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      state.volume = val;
      state.isMuted = false;
      updateVolumeUI();
      await invoke("set_volume", { volume: val });
    };

    els.volBarTrack.addEventListener("mousedown", (e) => {
      isVolChanging = true;
      volHandler(e);
    });
    window.addEventListener("mousemove", (e) => {
      if (isVolChanging) volHandler(e);
    });
    window.addEventListener("mouseup", () => {
      isVolChanging = false;
    });
  }

  if (els.btnMute) {
    let prevVolume = 0.8;
    els.btnMute.onclick = async () => {
      state.isMuted = !state.isMuted;
      if (state.isMuted) {
        prevVolume = state.volume > 0 ? state.volume : 0.8;
        state.volume = 0;
      } else {
        state.volume = prevVolume;
      }
      updateVolumeUI();
      await invoke("set_volume", { volume: state.volume });
    };
  }

  function updateVolumeUI() {
    const pct = `${state.volume * 100}%`;
    if (els.volBarFill) els.volBarFill.style.width = pct;
    if (els.volBarThumb) els.volBarThumb.style.left = pct;
    if (els.polyVol) els.polyVol.textContent = `vol: ${Math.round(state.volume * 100)}%`;
  }

  function updateTimelineUI() {
    const totalSec = state.currentTrack ? state.currentTrack.durationSec : 180;
    const curSec = state.positionMs / 1000;
    const pct = totalSec > 0 ? `${(curSec / totalSec) * 100}%` : "0%";

    if (els.timeCurrent) els.timeCurrent.textContent = formatTime(curSec);
    if (els.timeDuration) els.timeDuration.textContent = formatTime(totalSec);
    if (els.seekBarFill) els.seekBarFill.style.width = pct;
    if (els.seekBarThumb) els.seekBarThumb.style.left = pct;

    updateSyncedLyricsHighlight(state.positionMs);
  }

  // --- Lyrics Service & Sync ---
  async function fetchLyrics(track) {
    if (!els.lyricsLines) return;
    els.lyricsLines.innerHTML = '<div class="lyrics-loading">Поиск текста песни (LRCLIB)...</div>';

    try {
      const data = await invoke("get_lyrics", { track });
      state.lyrics = data;
      renderLyrics(data);
    } catch (e) {
      els.lyricsLines.innerHTML = '<div class="lyrics-none">Текст песни не найден.</div>';
    }
  }

  function renderLyrics(data) {
    if (!els.lyricsLines || !data) return;
    if (!data.lines || data.lines.length === 0) {
      els.lyricsLines.innerHTML = `<div class="lyrics-plain">${data.plainText || "Текст песни отсутствует."}</div>`;
      return;
    }

    els.lyricsLines.innerHTML = data.lines
      .map(
        (line, idx) => `
        <div class="lyrics-line" data-start="${line.startMs}" data-idx="${idx}">
          <span class="line-orig">${line.text}</span>
          ${line.translation ? `<span class="line-trans">${line.translation}</span>` : ""}
        </div>
      `
      )
      .join("");
  }

  function updateSyncedLyricsHighlight(curMs) {
    if (!els.lyricsLines || !state.lyrics || !state.lyrics.lines) return;
    const lines = els.lyricsLines.querySelectorAll(".lyrics-line");
    let activeLine = null;

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const start = parseInt(line.dataset.start, 10);
      const nextStart = i + 1 < lines.length ? parseInt(lines[i + 1].dataset.start, 10) : Infinity;

      if (curMs >= start && curMs < nextStart) {
        line.classList.add("active");
        activeLine = line;
      } else {
        line.classList.remove("active");
      }
    }

    if (activeLine) {
      activeLine.scrollIntoView({ behavior: "smooth", block: "center" });
    }
  }

  // Translation toggle
  if (els.toggleTranslate) {
    els.toggleTranslate.onchange = async (e) => {
      state.showTranslation = e.target.checked;
      if (state.showTranslation && state.lyrics && state.lyrics.lines) {
        const origStrings = state.lyrics.lines.map((l) => l.text);
        try {
          const trans = await invoke("translate_lyrics", {
            lines: origStrings,
            targetLang: "ru",
          });
          trans.forEach((t, i) => {
            if (state.lyrics.lines[i]) state.lyrics.lines[i].translation = t;
          });
          renderLyrics(state.lyrics);
        } catch (err) {
          console.error("Translation error:", err);
        }
      }
    };
  }

  // --- Queue View ---
  function renderQueueList() {
    if (!els.queueTracksList) return;
    if (state.queue.length === 0) {
      els.queueTracksList.innerHTML = '<div class="empty-notice">Очередь воспроизведения пуста.</div>';
      return;
    }

    els.queueTracksList.innerHTML = state.queue
      .map(
        (t, idx) => `
        <div class="queue-item chip ${idx === state.queueIndex ? "active" : ""}" data-idx="${idx}" style="display:flex; align-items:center; justify-content:space-between; padding:8px 12px; margin-bottom:4px; cursor:pointer;">
          <span style="font-size:12px; font-weight:700; width:24px;">${idx + 1}</span>
          <div style="flex:1; display:flex; flex-direction:column; min-width:0; overflow:hidden;">
            <span style="font-weight:700; font-size:13px; color:var(--text); white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">${t.title}</span>
            <span style="font-size:11px; color:var(--dim); white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">${t.artists ? t.artists.map(a => a.name).join(", ") : "—"}</span>
          </div>
          <button class="btn-icon btn-play-queue" data-idx="${idx}" style="background:none; border:none; color:var(--accent); font-size:15px; cursor:pointer;">▶</button>
        </div>
      `
      )
      .join("");

    els.queueTracksList.querySelectorAll(".queue-item").forEach((b) => {
      b.onclick = async () => {
        const idx = parseInt(b.dataset.idx, 10);
        await invoke("play_queue_index", { index: idx });
      };
    });
  }

  if (els.btnClearQueue) {
    els.btnClearQueue.onclick = async () => {
      state.queue = [];
      await invoke("clear_queue");
      renderQueueList();
    };
  }

  function renderNowPlayingInfo() {
    if (!els.nowPlayingLargeArt || !els.nowPlayingDetails) return;
    if (!state.currentTrack) {
      els.nowPlayingLargeArt.innerHTML = '<div class="empty-notice" style="height:200px; display:flex; align-items:center; justify-content:center;">Трек не выбран</div>';
      els.nowPlayingDetails.innerHTML = '';
      return;
    }
    const t = state.currentTrack;
    els.nowPlayingLargeArt.innerHTML = `
      <img src="${t.cover || ''}" style="width:100%; aspect-ratio:1; object-fit:cover; border-radius:var(--cover-radius); margin-bottom:14px; box-shadow:0 8px 24px rgba(0,0,0,0.3);">
    `;
    els.nowPlayingDetails.innerHTML = `
      <h2 style="font-size:18px; font-weight:800; margin-bottom:4px; color:var(--text);">${t.title}</h2>
      <p style="font-size:14px; color:var(--dim); margin-bottom:12px;">${t.artists ? t.artists.map(a => a.name).join(', ') : '—'}</p>
      <div style="font-size:12px; color:var(--mute); line-height:1.6;">
        <div>Альбом: ${t.album ? t.album.title : 'Сингл'}</div>
        <div>Длительность: ${formatTime(t.durationSec)}</div>
        <div>Источник: Hi-Fi Dynamic Stream</div>
      </div>
    `;
  }

  // --- Right Panel Tabs & Toggles ---
  els.rightTabs.forEach((tab) => {
    tab.addEventListener("click", () => {
      selectRightTab(tab.dataset.tab);
    });
  });

  function selectRightTab(tabName) {
    els.rightTabs.forEach((t) => {
      if (t.dataset.tab === tabName) t.classList.add("active");
      else t.classList.remove("active");
    });
    document.querySelectorAll(".tab-pane").forEach((p) => {
      p.classList.add("hidden");
      p.classList.remove("active");
    });
    const p = document.getElementById(`tab-${tabName}-content`);
    if (p) {
      p.classList.remove("hidden");
      p.classList.add("active");
    }
    if (tabName === "queue") renderQueueList();
    if (tabName === "info") renderNowPlayingInfo();
  }

  if (els.btnToggleLyrics) {
    els.btnToggleLyrics.onclick = () => {
      const activeTab = document.querySelector(".tab-btn.active")?.dataset.tab;
      if (!els.rightPanel.classList.contains("hidden") && activeTab === "lyrics") {
        els.rightPanel.classList.add("hidden");
      } else {
        els.rightPanel.classList.remove("hidden");
        selectRightTab("lyrics");
      }
    };
  }

  if (els.btnToggleQueue) {
    els.btnToggleQueue.onclick = () => {
      const activeTab = document.querySelector(".tab-btn.active")?.dataset.tab;
      if (!els.rightPanel.classList.contains("hidden") && activeTab === "queue") {
        els.rightPanel.classList.add("hidden");
      } else {
        els.rightPanel.classList.remove("hidden");
        selectRightTab("queue");
      }
    };
  }

  if (els.btnToggleNowplaying) {
    els.btnToggleNowplaying.onclick = () => {
      const activeTab = document.querySelector(".tab-btn.active")?.dataset.tab;
      if (!els.rightPanel.classList.contains("hidden") && activeTab === "info") {
        els.rightPanel.classList.add("hidden");
      } else {
        els.rightPanel.classList.remove("hidden");
        selectRightTab("info");
      }
    };
  }

  // --- 10-Bar FFT Spectrum (Levels Canvas 34x18px) ---
  const levelsCanvas = els.levelsCanvas;
  if (levelsCanvas) {
    const ctx = levelsCanvas.getContext("2d");
    const numBars = 10;
    const barWidth = 2;
    const gap = 1.4;
    let smoothLevels = new Array(numBars).fill(0.08);

    async function drawLevels() {
      if (state.isPlaying) {
        try {
          const bands = await invoke("get_spectrum");
          if (bands && bands.length >= 10) {
            for (let i = 0; i < numBars; i++) {
              const idx = Math.min(bands.length - 1, Math.floor(i * (bands.length / numBars)));
              const target = Math.max(0.08, Math.min(1.0, bands[idx]));
              smoothLevels[i] = smoothLevels[i] * 0.6 + target * 0.4;
            }
          }
        } catch (_) {}
      } else {
        for (let i = 0; i < numBars; i++) {
          smoothLevels[i] = Math.max(0.06, smoothLevels[i] * 0.85);
        }
      }

      ctx.clearRect(0, 0, levelsCanvas.width, levelsCanvas.height);
      const computedAccent = getComputedStyle(document.body).getPropertyValue("--accent").trim() || "#F4A7BB";
      const computedMute = getComputedStyle(document.body).getPropertyValue("--mute").trim() || "#8E8389";
      ctx.fillStyle = state.isPlaying ? computedAccent : computedMute;

      for (let i = 0; i < numBars; i++) {
        const h = Math.max(2, smoothLevels[i] * levelsCanvas.height);
        const x = i * (barWidth + gap);
        const y = levelsCanvas.height - h;
        ctx.beginPath();
        if (ctx.roundRect) {
          ctx.roundRect(x, y, barWidth, h, 1);
        } else {
          ctx.rect(x, y, barWidth, h);
        }
        ctx.fill();
      }

      requestAnimationFrame(drawLevels);
    }
    requestAnimationFrame(drawLevels);
  }

  // --- Periodic Playback State Polling ---
  setInterval(async () => {
    try {
      const pl = await invoke("get_playback_state");
      if (pl) {
        state.isPlaying = pl.isPlaying;
        state.positionMs = pl.positionMs;
        if (pl.currentTrack) {
          state.currentTrack = pl.currentTrack;
          updateTrackMetadataUI(pl.currentTrack);
        }
        if (pl.queue) {
          state.queue = pl.queue;
          state.queueIndex = pl.currentIndex;
        }
        updatePlayPauseUI();
        updateTimelineUI();
      }
    } catch (_) {}
  }, 250);

  // --- Initial Data Refresh ---
  async function refreshData() {
    try {
      const liked = await invoke("get_liked");
      if (liked) {
        state.likedTracks = liked;
        if (els.likedCount) els.likedCount.textContent = liked.length;
        const sub = document.getElementById("liked-subtitle");
        if (sub) sub.textContent = `${liked.length} треков`;
      }
      const history = await invoke("get_history");
      if (history) state.history = history;

      const playlists = await invoke("get_playlists");
      if (playlists) {
        state.playlists = playlists;
        renderNavPlaylists();
      }

      const settings = await invoke("get_settings");
      if (settings) {
        applyTheme(settings.theme || "tategaki", settings.themeDay ? "day" : "night", "glass");
        state.volume = settings.volume || 0.8;
        updateVolumeUI();
      }
    } catch (e) {
      console.warn("Initial data load error:", e);
    }
  }

  function renderNavPlaylists() {
    if (!els.navPlaylistsList) return;
    els.navPlaylistsList.innerHTML = state.playlists
      .map(
        (pl) => `
        <div class="lib-item nav-subitem" data-id="${pl.id}" style="padding:6px 10px; font-size:12.5px;">
          <span style="color:var(--accent); margin-right:8px;">♫</span>
          <span style="font-weight:600; color:var(--text);">${pl.name}</span>
        </div>
      `
      )
      .join("");
  }

  // --- Modals: Theme ---
  if (els.btnThemeSelector && els.modalTheme) {
    els.btnThemeSelector.onclick = () => els.modalTheme.classList.remove("hidden");
  }
  if (els.modalThemeClose && els.modalTheme) {
    els.modalThemeClose.onclick = () => els.modalTheme.classList.add("hidden");
  }
  document.querySelectorAll("#modal-theme .theme-card").forEach((card) => {
    card.addEventListener("click", async () => {
      const theme = card.dataset.theme;
      const mode = document.querySelector('input[name="theme-mode"]:checked')?.value || "night";
      applyTheme(theme, mode, state.look);
      els.modalTheme.classList.add("hidden");

      try {
        const settings = await invoke("get_settings");
        if (settings) {
          settings.theme = theme;
          settings.themeDay = mode === "day";
          await invoke("save_settings", { settings });
        }
      } catch (_) {}
    });
  });

  // --- Modals: Sleep Timer ---
  if (els.btnSleepTimer && els.modalSleepTimer) {
    els.btnSleepTimer.onclick = () => els.modalSleepTimer.classList.remove("hidden");
  }
  if (els.modalSleepClose && els.modalSleepTimer) {
    els.modalSleepClose.onclick = () => els.modalSleepTimer.classList.add("hidden");
  }
  let sleepTimeout = null;
  document.querySelectorAll(".btn-sleep-opt").forEach((btn) => {
    btn.onclick = () => {
      const mins = parseInt(btn.dataset.mins, 10);
      if (sleepTimeout) clearTimeout(sleepTimeout);
      if (mins > 0) {
        sleepTimeout = setTimeout(async () => {
          if (state.isPlaying) {
            state.isPlaying = false;
            updatePlayPauseUI();
            await invoke("toggle_play_pause");
          }
        }, mins * 60 * 1000);
      }
      els.modalSleepTimer.classList.add("hidden");
    };
  });

  // --- Modals: Phone Sync (Backup Import) ---
  if (els.btnPhoneSync && els.modalPhoneSync) {
    els.btnPhoneSync.onclick = () => els.modalPhoneSync.classList.remove("hidden");
  }
  if (els.modalBackupClose && els.modalPhoneSync) {
    els.modalBackupClose.onclick = () => els.modalPhoneSync.classList.add("hidden");
  }
  const btnImportBackup = document.getElementById("btn-import-backup-submit");
  const backupInput = document.getElementById("backup-file-path");
  const backupStatus = document.getElementById("backup-status");
  if (btnImportBackup && backupInput) {
    btnImportBackup.onclick = async () => {
      const path = backupInput.value.trim();
      if (!path) return;
      if (backupStatus) backupStatus.textContent = "Импорт...";
      try {
        const count = await invoke("import_backup", { zipPath: path });
        if (backupStatus) backupStatus.textContent = `Успешно импортировано ${count} треков!`;
        refreshData();
      } catch (err) {
        if (backupStatus) backupStatus.textContent = `Ошибка: ${err}`;
      }
    };
  }

  // --- Modals: New Playlist ---
  if (els.btnNewPlaylist && els.modalNewPlaylist) {
    els.btnNewPlaylist.onclick = () => els.modalNewPlaylist.classList.remove("hidden");
  }
  if (els.modalPlaylistClose && els.modalNewPlaylist) {
    els.modalPlaylistClose.onclick = () => els.modalNewPlaylist.classList.add("hidden");
  }
  const btnCreatePlaylist = document.getElementById("btn-create-playlist-submit");
  const playlistInput = document.getElementById("new-playlist-name");
  if (btnCreatePlaylist && playlistInput) {
    btnCreatePlaylist.onclick = async () => {
      const name = playlistInput.value.trim();
      if (!name) return;
      try {
        await invoke("create_playlist", { name });
        playlistInput.value = "";
        els.modalNewPlaylist.classList.add("hidden");
        refreshData();
      } catch (err) {
        console.error("Create playlist error:", err);
      }
    };
  }

  // Init
  refreshData();
  navigateTo("home");
})();
