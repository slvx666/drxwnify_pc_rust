/**
 * Drxwnify Full Edition — Frontend Application Logic
 */

(function () {
  // Tauri IPC helper
  const invoke = window.__TAURI__ ? window.__TAURI__.core.invoke : async (cmd, args) => {
    console.log(`[IPC Mock] ${cmd}`, args);
    return null;
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
    btnBack: document.getElementById("nav-back"),
    btnForward: document.getElementById("nav-forward"),
    btnRefresh: document.getElementById("nav-refresh"),
    searchInput: document.getElementById("search-input"),
    searchClear: document.getElementById("search-clear"),
    btnThemeSelector: document.getElementById("btn-theme-selector"),
    modalTheme: document.getElementById("modal-theme"),
    modalThemeClose: document.getElementById("modal-theme-close"),
    btnMin: document.getElementById("btn-minimize"),
    btnMax: document.getElementById("btn-maximize"),
    btnClose: document.getElementById("btn-close"),
    navItems: document.querySelectorAll(".nav-item"),
    likedCount: document.getElementById("liked-count"),
    navPlaylistsList: document.getElementById("nav-playlists-list"),
    btnNewPlaylist: document.getElementById("btn-new-playlist"),
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
    spectrogramCanvas: document.getElementById("spectrogram-canvas"),
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

    els.navItems.forEach((item) => {
      if (item.dataset.screen === screenId) item.classList.add("active");
      else item.classList.remove("active");
    });

    document.querySelectorAll(".screen-view").forEach((s) => s.classList.add("hidden"));
    let target = document.getElementById(`screen-${screenId}`);
    if (!target) {
      target = document.getElementById("screen-home");
    }
    target.classList.remove("hidden");
    renderScreen(screenId, target);
  }

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

  els.navItems.forEach((item) => {
    item.addEventListener("click", () => {
      const scr = item.dataset.screen;
      if (scr) navigateTo(scr);
    });
  });

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
          <div class="quick-picks-grid" id="quick-picks-grid">
            <div class="pick-card" data-action="liked">
              <div class="pick-icon">♥</div>
              <div class="pick-name">Любимые треки</div>
            </div>
            <div class="pick-card" data-action="history">
              <div class="pick-icon">⏱</div>
              <div class="pick-name">История прослушиваний</div>
            </div>
            <div class="pick-card" data-action="downloads">
              <div class="pick-icon">⬇</div>
              <div class="pick-name">Загрузки</div>
            </div>
          </div>
        </div>

        <div class="section-block">
          <div class="section-title">Недавно играли</div>
          <div class="recent-tracks-list" id="home-recent-tracks">
            ${
              state.history.length === 0
                ? '<div class="empty-notice">Здесь появятся ваши недавно воспроизведённые треки.</div>'
                : state.history
                    .slice(0, 5)
                    .map(
                      (p) => `
              <div class="track-row" data-id="${p.track.id}">
                <div class="track-left">
                  <div class="track-thumb">${p.track.cover ? `<img src="${p.track.cover}">` : "♪"}</div>
                  <div class="track-meta">
                    <span class="track-name">${p.track.title}</span>
                    <span class="track-artists">${p.track.artists ? p.track.artists.map(a => a.name).join(", ") : "—"}</span>
                  </div>
                </div>
                <div class="track-actions">
                  <button class="btn-icon btn-play-row" data-id="${p.track.id}">▶</button>
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
      btn.onclick = () => {
        const tr = state.history.find((p) => p.track.id === btn.dataset.id);
        if (tr) playTrack(tr.track);
      };
    });
  }

  function renderLikedScreen(container) {
    container.innerHTML = `
      <div class="screen-content">
        <div class="playlist-header">
          <div class="playlist-cover-large">♥</div>
          <div class="playlist-info">
            <span class="playlist-type">ПЛЕЙЛИСТ</span>
            <h1 class="playlist-name">Любимые треки</h1>
            <span class="playlist-meta">${state.likedTracks.length} треков</span>
            <div class="playlist-buttons">
              <button id="btn-play-liked-all" class="btn-primary">Слушать всё</button>
            </div>
          </div>
        </div>

        <div class="tracks-table">
          <div class="table-header">
            <span class="col-num">#</span>
            <span class="col-title">НАЗВАНИЕ</span>
            <span class="col-album">АЛЬБОМ</span>
            <span class="col-time">ДЛИТЕЛЬНОСТЬ</span>
          </div>
          <div id="liked-rows-container">
            ${
              state.likedTracks.length === 0
                ? '<div class="empty-notice">В любимых пока нет треков. Нажмите на сердечко во время воспроизведения!</div>'
                : state.likedTracks
                    .map(
                      (t, idx) => `
                <div class="table-row ${state.currentTrack && state.currentTrack.id === t.id ? "playing" : ""}" data-id="${t.id}">
                  <span class="col-num">${idx + 1}</span>
                  <div class="col-title">
                    <div class="track-thumb-small">${t.cover ? `<img src="${t.cover}">` : "♪"}</div>
                    <div class="track-title-info">
                      <span class="t-name">${t.title}</span>
                      <span class="t-artist">${t.artists ? t.artists.map(a => a.name).join(", ") : "—"}</span>
                    </div>
                  </div>
                  <span class="col-album">${t.album ? t.album.name : "—"}</span>
                  <span class="col-time">${formatTime(t.durationSec)}</span>
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
        const tr = state.likedTracks.find((t) => t.id === r.dataset.id);
        if (tr) playTrack(tr);
      };
    });
  }

  function renderSearchScreen(container) {
    container.innerHTML = `
      <div class="screen-content">
        <h2>Поиск по источникам (YouTube Music, VK, Lossless Hi-Fi)</h2>
        <div id="search-results-area">
          <div class="empty-notice">Введите поисковый запрос в верхнюю панель или нажмите Enter.</div>
        </div>
      </div>
    `;
  }

  function renderDownloadsScreen(container) {
    container.innerHTML = `
      <div class="screen-content">
        <div class="playlist-header">
          <div class="playlist-cover-large">⬇</div>
          <div class="playlist-info">
            <span class="playlist-type">ОФЛАЙН МЕДИАТЕКА</span>
            <h1 class="playlist-name">Загруженные треки</h1>
            <span class="playlist-meta">Хранятся в %USERPROFILE%\\Music\\Drxwnify</span>
          </div>
        </div>
        <div class="empty-notice">Загруженные локальные треки готовы к воспроизведению без подключения к сети.</div>
      </div>
    `;
  }

  function renderHistoryScreen(container) {
    container.innerHTML = `
      <div class="screen-content">
        <div class="history-top">
          <h1>История прослушиваний</h1>
          <button id="btn-clear-history" class="btn-text">Очистить историю</button>
        </div>
        <div class="history-list">
          ${
            state.history.length === 0
              ? '<div class="empty-notice">История пуста.</div>'
              : state.history
                  .map(
                    (p) => `
              <div class="track-row" data-id="${p.track.id}">
                <div class="track-left">
                  <div class="track-thumb">${p.track.cover ? `<img src="${p.track.cover}">` : "♪"}</div>
                  <div class="track-meta">
                    <span class="track-name">${p.track.title}</span>
                    <span class="track-artists">${p.track.artists ? p.track.artists.map(a => a.name).join(", ") : "—"}</span>
                  </div>
                </div>
                <div class="track-time">${new Date(p.timestamp).toLocaleTimeString()}</div>
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
  }

  function renderFriendsScreen(container) {
    const friendList = state.friends.length
      ? state.friends
      : [
          { pubkey: "slvx666", name: "slvx666 (drxwnslvt)", isOnline: true, currentTrack: { title: "Drown", artistNames: () => "Bring Me The Horizon" } },
          { pubkey: "fend1x", name: "fend1x", isOnline: true, currentTrack: { title: "Resonance", artistNames: () => "HOME" } },
          { pubkey: "clvd", name: "clavdiy", isOnline: false, currentTrack: null },
        ];

    const activeFriend = state.activeChatFriend || friendList[0];

    container.innerHTML = `
      <div class="chathub-container">
        <div class="chathub-sidebar">
          <div class="chathub-header">
            <h3>Друзья & Nostr</h3>
            <span class="badge">Relays: 5 online</span>
          </div>
          <div class="chathub-friends-list">
            ${friendList
              .map(
                (f) => `
              <div class="chathub-friend-item ${activeFriend.pubkey === f.pubkey ? "active" : ""}" data-pubkey="${f.pubkey}">
                <div class="chathub-avatar">${f.name.charAt(0).toUpperCase()}</div>
                <div class="chathub-friend-info">
                  <span class="friend-name">${f.name}</span>
                  <span class="friend-status">${f.currentTrack ? `♫ ${f.currentTrack.title}` : (f.isOnline ? "В сети" : "Не в сети")}</span>
                </div>
              </div>
            `
              )
              .join("")}
          </div>
        </div>

        <div class="chathub-chat-pane">
          <div class="chat-header">
            <span class="chat-title">${activeFriend.name}</span>
            <span class="chat-meta">Nostr Kind 4 Direct Message</span>
          </div>
          <div class="chat-messages" id="chat-messages-container">
            <div class="chat-bubble incoming">
              <div class="msg-text">Привет! Слушаешь новый релиз?</div>
              <div class="msg-time">14:20</div>
            </div>
            <div class="chat-bubble outgoing">
              <div class="msg-text">Да, слушаю через Drxwnify Full на Rust! Звучит шикарно.</div>
              <div class="msg-time">14:22</div>
            </div>
          </div>
          <div class="chat-input-bar">
            <input type="text" id="chat-text-input" placeholder="Написать сообщение через Nostr...">
            <button id="btn-show-stickers" class="btn-icon" title="Стикеры">☺</button>
            <button id="btn-chat-send" class="btn-primary">Отправить</button>
          </div>
          <div id="stickers-picker" class="stickers-picker hidden">
            <div class="stickers-grid">
              <img src="stickers/sweet_bird.webp" class="sticker-thumb" data-name="sweet_bird">
              <img src="stickers/kitty_please.webp" class="sticker-thumb" data-name="kitty_please">
              <img src="stickers/smart_man.webp" class="sticker-thumb" data-name="smart_man">
              <img src="stickers/happy_guy.webp" class="sticker-thumb" data-name="happy_guy">
              <img src="stickers/crying_man.webp" class="sticker-thumb" data-name="crying_man">
              <img src="stickers/writing_man.webp" class="sticker-thumb" data-name="writing_man">
              <img src="stickers/fans.webp" class="sticker-thumb" data-name="fans">
              <img src="stickers/super_man.webp" class="sticker-thumb" data-name="super_man">
            </div>
          </div>
        </div>
      </div>
    `;

    // Friend selection
    container.querySelectorAll(".chathub-friend-item").forEach((it) => {
      it.onclick = () => {
        state.activeChatFriend = friendList.find((f) => f.pubkey === it.dataset.pubkey);
        renderFriendsScreen(container);
      };
    });

    // Sticker picker toggle
    const stickerBtn = container.querySelector("#btn-show-stickers");
    const stickerPicker = container.querySelector("#stickers-picker");
    if (stickerBtn && stickerPicker) {
      stickerBtn.onclick = () => stickerPicker.classList.toggle("hidden");
    }

    // Send sticker
    container.querySelectorAll(".sticker-thumb").forEach((s) => {
      s.onclick = () => {
        const msgContainer = container.querySelector("#chat-messages-container");
        if (msgContainer) {
          msgContainer.innerHTML += `
            <div class="chat-bubble outgoing">
              <img src="${s.src}" class="msg-sticker">
              <div class="msg-time">${new Date().toLocaleTimeString()}</div>
            </div>
          `;
          msgContainer.scrollTop = msgContainer.scrollHeight;
        }
        stickerPicker.classList.add("hidden");
      };
    });

    // Send text message
    const sendBtn = container.querySelector("#btn-chat-send");
    const input = container.querySelector("#chat-text-input");
    if (sendBtn && input) {
      sendBtn.onclick = () => {
        const text = input.value.trim();
        if (!text) return;
        const msgContainer = container.querySelector("#chat-messages-container");
        if (msgContainer) {
          msgContainer.innerHTML += `
            <div class="chat-bubble outgoing">
              <div class="msg-text">${text}</div>
              <div class="msg-time">${new Date().toLocaleTimeString()}</div>
            </div>
          `;
          msgContainer.scrollTop = msgContainer.scrollHeight;
        }
        input.value = "";
      };
      input.onkeydown = (e) => {
        if (e.key === "Enter") sendBtn.click();
      };
    }
  }

  function renderReviewsScreen(container) {
    container.innerHTML = `
      <div class="screen-content">
        <h1>Обзоры drxwnslvt</h1>
        <p class="section-desc">Авторские рецензии и подборки от создателя Drxwnify.</p>
        <div class="reviews-grid">
          <div class="review-card">
            <div class="review-header">
              <span class="review-album">Sempiternal</span>
              <span class="review-rating">★ 9.8 / 10</span>
            </div>
            <p class="review-body">Монументальный релиз, определивший звучание альтернативной сцены на целое десятилетие.</p>
          </div>
          <div class="review-card">
            <div class="review-header">
              <span class="review-album">Holy Hell</span>
              <span class="review-rating">★ 9.5 / 10</span>
            </div>
            <p class="review-body">Искренняя эмоциональная ярость и потрясающая звуковая глубина оркестровых аранжировок.</p>
          </div>
        </div>
      </div>
    `;
  }

  // --- Search Service Trigger ---
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
      }, 350);
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
    container.innerHTML = '<div class="loading-spinner">Поиск по источникам...</div>';

    try {
      const res = await invoke("search", { query });
      if (!res || !res.tracks || res.tracks.length === 0) {
        container.innerHTML = `<div class="empty-notice">По запросу "${query}" ничего не найдено.</div>`;
        return;
      }

      container.innerHTML = `
        <div class="search-tracks-list">
          ${res.tracks
            .map(
              (t) => `
            <div class="track-row" data-id="${t.id}">
              <div class="track-left">
                <div class="track-thumb">${t.cover ? `<img src="${t.cover}">` : "♪"}</div>
                <div class="track-meta">
                  <span class="track-name">${t.title}</span>
                  <span class="track-artists">${t.artists ? t.artists.map(a => a.name).join(", ") : "—"}</span>
                </div>
              </div>
              <div class="track-time">${formatTime(t.durationSec)}</div>
              <div class="track-actions">
                <button class="btn-icon btn-play-search" data-id="${t.id}">▶</button>
              </div>
            </div>
          `
            )
            .join("")}
        </div>
      `;

      container.querySelectorAll(".btn-play-search").forEach((b) => {
        b.onclick = () => {
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

  // --- Volume ---
  if (els.volBarTrack) {
    let isVolChanging = false;
    const volHandler = async (e) => {
      const rect = els.volBarTrack.getBoundingClientRect();
      const val = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      state.volume = val;
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
        <div class="queue-item ${idx === state.queueIndex ? "active" : ""}" data-idx="${idx}">
          <span class="q-num">${idx + 1}</span>
          <div class="q-meta">
            <span class="q-title">${t.title}</span>
            <span class="q-artist">${t.artists ? t.artists.map(a => a.name).join(", ") : "—"}</span>
          </div>
          <button class="btn-icon btn-play-queue" data-idx="${idx}">▶</button>
        </div>
      `
      )
      .join("");

    els.queueTracksList.querySelectorAll(".btn-play-queue").forEach((b) => {
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

  // --- Right Panel Tabs ---
  els.rightTabs.forEach((tab) => {
    tab.addEventListener("click", () => {
      els.rightTabs.forEach((t) => t.classList.remove("active"));
      tab.classList.add("active");
      const name = tab.dataset.tab;
      document.querySelectorAll(".tab-pane").forEach((p) => p.classList.add("hidden"));
      const p = document.getElementById(`tab-${name}-content`);
      if (p) p.classList.remove("hidden");
      if (name === "queue") renderQueueList();
    });
  });

  // --- 32-Band Spectrum Analyzer Canvas ---
  const specCanvas = els.spectrogramCanvas;
  if (specCanvas) {
    const specCtx = specCanvas.getContext("2d");
    const numBands = 32;

    async function drawSpectrum() {
      if (state.isPlaying) {
        try {
          const bands = await invoke("get_spectrum");
          if (bands && bands.length === numBands) {
            specCtx.clearRect(0, 0, specCanvas.width, specCanvas.height);
            const barWidth = specCanvas.width / numBands;

            for (let i = 0; i < numBands; i++) {
              const h = bands[i] * specCanvas.height;
              const y = specCanvas.height - h;
              specCtx.fillStyle = `hsl(${280 + i * 2}, 85%, 65%)`;
              specCtx.fillRect(i * barWidth, y, barWidth - 1, h);
            }
          }
        } catch (_) {}
      } else {
        specCtx.clearRect(0, 0, specCanvas.width, specCanvas.height);
      }
      requestAnimationFrame(drawSpectrum);
    }
    requestAnimationFrame(drawSpectrum);
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
        <div class="nav-subitem" data-id="${pl.id}">
          <span class="subitem-icon">♫</span>
          <span class="subitem-name">${pl.name}</span>
        </div>
      `
      )
      .join("");
  }

  // --- Theme Modal ---
  if (els.btnThemeSelector && els.modalTheme) {
    els.btnThemeSelector.onclick = () => els.modalTheme.classList.remove("hidden");
  }
  if (els.modalThemeClose && els.modalTheme) {
    els.modalThemeClose.onclick = () => els.modalTheme.classList.add("hidden");
  }
  document.querySelectorAll(".theme-card").forEach((card) => {
    card.addEventListener("click", async () => {
      const theme = card.dataset.theme;
      const mode = document.querySelector('input[name="theme-mode"]:checked')?.value || "night";
      applyTheme(theme, mode, state.look);
      els.modalTheme.classList.add("hidden");

      // Save to settings
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

  // Init
  refreshData();
  navigateTo("home");
})();
