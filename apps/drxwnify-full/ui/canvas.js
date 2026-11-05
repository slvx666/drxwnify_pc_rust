/**
 * Drxwnify Full Edition — Procedural Canvas Background Engine
 * Renders 9 unique themed procedural backdrops with AnimationGate:
 * - 30 FPS active focused
 * - 12 FPS background unfocused
 * - 0 FPS minimized / hidden
 */

(function () {
  const canvas = document.getElementById("backdrop-canvas") || document.getElementById("bg-canvas");
  if (!canvas) return;
  const ctx = canvas.getContext("2d");

  let width = (canvas.width = window.innerWidth);
  let height = (canvas.height = window.innerHeight);

  window.addEventListener("resize", () => {
    width = canvas.width = window.innerWidth;
    height = canvas.height = window.innerHeight;
  });

  let currentTheme = "Tategaki";
  let isWindowFocused = document.hasFocus();
  let isVisible = !document.hidden;
  let isRunning = true;
  let lastFrameTime = 0;

  // Window Focus & Visibility listeners
  window.addEventListener("focus", () => {
    isWindowFocused = true;
  });
  window.addEventListener("blur", () => {
    isWindowFocused = false;
  });
  document.addEventListener("visibilitychange", () => {
    isVisible = !document.hidden;
  });

  // Particle systems
  const particles = [];
  const MAX_PARTICLES = 70;

  function initParticles() {
    particles.length = 0;
    for (let i = 0; i < MAX_PARTICLES; i++) {
      particles.push({
        x: Math.random() * width,
        y: Math.random() * height,
        vx: (Math.random() - 0.5) * 1.2,
        vy: (Math.random() - 0.5) * 1.2,
        size: Math.random() * 3 + 1.5,
        alpha: Math.random() * 0.7 + 0.3,
        angle: Math.random() * Math.PI * 2,
        rotSpeed: (Math.random() - 0.5) * 0.04,
        hue: Math.random() * 20 - 10,
      });
    }
  }
  initParticles();

  // Retro Grid State
  let retroOffset = 0;

  // 1. Mist (Tategaki)
  function renderMist(time) {
    ctx.clearRect(0, 0, width, height);
    for (let layer = 0; layer < 4; layer++) {
      const speed = (layer + 1) * 0.0004;
      const t = time * speed;
      ctx.beginPath();
      ctx.moveTo(0, height);
      for (let x = 0; x <= width; x += 30) {
        const y =
          height * 0.55 +
          layer * 45 +
          Math.sin(x * 0.003 + t) * 35 +
          Math.cos(x * 0.007 - t * 0.8) * 20;
        ctx.lineTo(x, y);
      }
      ctx.lineTo(width, height);
      ctx.closePath();
      const alpha = 0.04 + layer * 0.03;
      ctx.fillStyle = `rgba(180, 190, 205, ${alpha})`;
      ctx.fill();
    }
  }

  // 2. Gruvbox (Warm Particles & Graph)
  function renderGruvbox(time) {
    ctx.clearRect(0, 0, width, height);
    ctx.fillStyle = "rgba(214, 93, 14, 0.4)";
    ctx.strokeStyle = "rgba(250, 189, 47, 0.12)";

    for (let i = 0; i < particles.length; i++) {
      const p = particles[i];
      p.x += p.vx * 0.6;
      p.y += p.vy * 0.6;

      if (p.x < 0) p.x = width;
      if (p.x > width) p.x = 0;
      if (p.y < 0) p.y = height;
      if (p.y > height) p.y = 0;

      ctx.beginPath();
      ctx.arc(p.x, p.y, p.size * 0.8, 0, Math.PI * 2);
      ctx.fill();

      for (let j = i + 1; j < particles.length; j++) {
        const p2 = particles[j];
        const dx = p.x - p2.x;
        const dy = p.y - p2.y;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist < 110) {
          ctx.beginPath();
          ctx.moveTo(p.x, p.y);
          ctx.lineTo(p2.x, p2.y);
          ctx.stroke();
        }
      }
    }
  }

  // 3. Retro (Retrowave 3D Perspective Grid & Sun)
  function renderRetro(time) {
    ctx.clearRect(0, 0, width, height);
    const horizon = height * 0.55;

    // Glowing Neon Sun
    const sunRadius = Math.min(width, height) * 0.18;
    const sunGradient = ctx.createLinearGradient(0, horizon - sunRadius * 2, 0, horizon);
    sunGradient.addColorStop(0, "rgba(255, 230, 0, 0.8)");
    sunGradient.addColorStop(0.5, "rgba(255, 0, 128, 0.7)");
    sunGradient.addColorStop(1, "rgba(255, 0, 128, 0.0)");

    ctx.save();
    ctx.beginPath();
    ctx.arc(width * 0.5, horizon - 20, sunRadius, 0, Math.PI * 2);
    ctx.fillStyle = sunGradient;
    ctx.shadowColor = "#ff007f";
    ctx.shadowBlur = 25;
    ctx.fill();
    ctx.restore();

    // Perspective Grid
    retroOffset = (retroOffset + 1.2) % 40;
    ctx.strokeStyle = "rgba(0, 240, 255, 0.28)";
    ctx.lineWidth = 1;

    // Longitudinal vanishing lines
    const fov = width * 0.5;
    for (let x = -width; x <= width * 2; x += 60) {
      ctx.beginPath();
      ctx.moveTo(width * 0.5, horizon);
      ctx.lineTo(x, height);
      ctx.stroke();
    }

    // Transverse lines
    for (let d = 1; d <= 25; d++) {
      const z = (d * 40 - retroOffset);
      if (z <= 0) continue;
      const y = horizon + (height - horizon) * (z / 1000);
      if (y > height) continue;
      const alpha = Math.min(1.0, (y - horizon) / (height - horizon));
      ctx.strokeStyle = `rgba(255, 0, 128, ${alpha * 0.35})`;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }
  }

  // 4. Depth (Oceanic Swell & Light Beams)
  function renderDepth(time) {
    ctx.clearRect(0, 0, width, height);

    // Light refractions
    ctx.save();
    const beamGrad = ctx.createLinearGradient(0, 0, width, height);
    beamGrad.addColorStop(0, "rgba(0, 210, 255, 0.06)");
    beamGrad.addColorStop(0.5, "rgba(0, 100, 200, 0.02)");
    beamGrad.addColorStop(1, "transparent");
    ctx.fillStyle = beamGrad;
    ctx.fillRect(0, 0, width, height);
    ctx.restore();

    // Water wave layers
    for (let i = 0; i < 3; i++) {
      ctx.beginPath();
      ctx.moveTo(0, height);
      const waveSpeed = time * 0.0006 + i * 1.5;
      for (let x = 0; x <= width; x += 25) {
        const y =
          height * 0.65 +
          i * 35 +
          Math.sin(x * 0.005 + waveSpeed) * 25 +
          Math.cos(x * 0.002 - waveSpeed * 0.5) * 15;
        ctx.lineTo(x, y);
      }
      ctx.lineTo(width, height);
      ctx.closePath();
      ctx.fillStyle = `rgba(0, 150, 255, ${0.05 + i * 0.03})`;
      ctx.fill();
    }
  }

  // 5. Sakura (Falling Petals)
  function renderSakura(time) {
    ctx.clearRect(0, 0, width, height);
    ctx.fillStyle = "rgba(255, 183, 197, 0.65)";

    for (let p of particles) {
      p.y += (p.vy.abs ? Math.abs(p.vy) : 0.8) + 0.6;
      p.x += Math.sin(p.y * 0.01 + time * 0.001) * 1.2;
      p.angle += p.rotSpeed;

      if (p.y > height + 20) {
        p.y = -20;
        p.x = Math.random() * width;
      }
      if (p.x > width + 20) p.x = -20;
      if (p.x < -20) p.x = width + 20;

      ctx.save();
      ctx.translate(p.x, p.y);
      ctx.rotate(p.angle);
      ctx.beginPath();
      ctx.ellipse(0, 0, p.size * 2, p.size, 0, 0, Math.PI * 2);
      ctx.fill();
      ctx.restore();
    }
  }

  // 6. Cozy (Fireplace Embers Floating Up)
  function renderCozy(time) {
    ctx.clearRect(0, 0, width, height);

    for (let p of particles) {
      p.y -= Math.abs(p.vy) * 0.9 + 0.8;
      p.x += Math.sin(p.y * 0.02 + time * 0.002) * 0.7;

      if (p.y < -20) {
        p.y = height + 20;
        p.x = Math.random() * width;
      }

      const grad = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, p.size * 2);
      grad.addColorStop(0, "rgba(255, 170, 50, 0.8)");
      grad.addColorStop(0.5, "rgba(240, 70, 20, 0.4)");
      grad.addColorStop(1, "transparent");

      ctx.beginPath();
      ctx.arc(p.x, p.y, p.size * 2, 0, Math.PI * 2);
      ctx.fillStyle = grad;
      ctx.fill();
    }
  }

  // 7. StarAtlas (Observatory Constellations)
  function renderStarAtlas(time) {
    ctx.clearRect(0, 0, width, height);
    ctx.fillStyle = "rgba(245, 245, 255, 0.75)";
    ctx.strokeStyle = "rgba(100, 180, 255, 0.12)";

    for (let i = 0; i < particles.length; i++) {
      const p = particles[i];
      p.x += p.vx * 0.2;
      p.y += p.vy * 0.2;

      if (p.x < 0) p.x = width;
      if (p.x > width) p.x = 0;
      if (p.y < 0) p.y = height;
      if (p.y > height) p.y = 0;

      const twinkle = Math.sin(time * 0.003 + i) * 0.3 + 0.7;
      ctx.beginPath();
      ctx.arc(p.x, p.y, p.size * twinkle * 0.7, 0, Math.PI * 2);
      ctx.fill();

      for (let j = i + 1; j < particles.length; j++) {
        const p2 = particles[j];
        const dist = Math.hypot(p.x - p2.x, p.y - p2.y);
        if (dist < 90) {
          ctx.beginPath();
          ctx.moveTo(p.x, p.y);
          ctx.lineTo(p2.x, p2.y);
          ctx.stroke();
        }
      }
    }
  }

  // 8. Ash (Satanic Crimson & Dark Ash)
  function renderAsh(time) {
    ctx.clearRect(0, 0, width, height);

    // Center Demonic Sigil Glow
    const cx = width * 0.5;
    const cy = height * 0.5;
    const pulse = Math.sin(time * 0.002) * 0.03 + 0.06;
    const runeGrad = ctx.createRadialGradient(cx, cy, 20, cx, cy, height * 0.45);
    runeGrad.addColorStop(0, `rgba(220, 20, 40, ${pulse * 1.5})`);
    runeGrad.addColorStop(1, "transparent");
    ctx.fillStyle = runeGrad;
    ctx.fillRect(0, 0, width, height);

    // Ash specks rising
    for (let p of particles) {
      p.y -= Math.abs(p.vy) * 0.7 + 0.6;
      p.x += (Math.random() - 0.5) * 0.8;

      if (p.y < -20) {
        p.y = height + 20;
        p.x = Math.random() * width;
      }

      ctx.beginPath();
      ctx.arc(p.x, p.y, p.size * 0.9, 0, Math.PI * 2);
      ctx.fillStyle = p.hue > 0 ? "rgba(220, 30, 40, 0.6)" : "rgba(40, 40, 45, 0.7)";
      ctx.fill();
    }
  }

  // 9. Mono (Minimal Scanlines & Subtle Noise)
  function renderMono(time) {
    ctx.clearRect(0, 0, width, height);
    ctx.fillStyle = "rgba(255, 255, 255, 0.015)";

    // Floating subtle scanline
    const scanY = (time * 0.06) % height;
    ctx.fillStyle = "rgba(255, 255, 255, 0.04)";
    ctx.fillRect(0, scanY, width, 2);

    // Clean particles
    ctx.fillStyle = "rgba(240, 240, 240, 0.25)";
    for (let p of particles.slice(0, 30)) {
      p.x += p.vx * 0.3;
      p.y += p.vy * 0.3;
      if (p.x < 0) p.x = width;
      if (p.x > width) p.x = 0;
      if (p.y < 0) p.y = height;
      if (p.y > height) p.y = 0;

      ctx.beginPath();
      ctx.arc(p.x, p.y, 1.2, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  // Main Render Dispatcher
  function render(time) {
    switch (currentTheme.toLowerCase()) {
      case "gruvbox":
        renderGruvbox(time);
        break;
      case "retrowave":
      case "retro":
        renderRetro(time);
        break;
      case "depth":
        renderDepth(time);
        break;
      case "sakura":
        renderSakura(time);
        break;
      case "cozy":
        renderCozy(time);
        break;
      case "observatory":
      case "staratlas":
        renderStarAtlas(time);
        break;
      case "satanic":
      case "ash":
        renderAsh(time);
        break;
      case "mono":
        renderMono(time);
        break;
      case "tategaki":
      default:
        renderMist(time);
        break;
    }
  }

  // AnimationGate Loop
  function loop(timestamp) {
    if (!isRunning) return;

    // Check window state
    if (!isVisible) {
      // 0 FPS when minimized/hidden: wait 250ms and check again
      setTimeout(() => requestAnimationFrame(loop), 250);
      return;
    }

    const targetFps = isWindowFocused ? 30 : 12;
    const frameInterval = 1000 / targetFps;
    const elapsed = timestamp - lastFrameTime;

    if (elapsed >= frameInterval) {
      lastFrameTime = timestamp - (elapsed % frameInterval);
      render(timestamp);
    }

    requestAnimationFrame(loop);
  }

  requestAnimationFrame(loop);

  // Global API
  window.CanvasBackground = {
    setTheme(name) {
      currentTheme = name;
    },
    start() {
      if (!isRunning) {
        isRunning = true;
        requestAnimationFrame(loop);
      }
    },
    stop() {
      isRunning = false;
    },
  };
})();
