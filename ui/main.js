// ScribeRx - Apple Siri Liquid Aurora Visualizer

class ScribeRxUI {
  constructor() {
    this.currentState = 'idle';
    this.micLevel = 0.0;
    this.phase = 0.0;
    this.tokens = [];
    this.selectedAlternatives = {};

    this.initCanvas();
    this.initTauriListeners();
    this.initKeyboardListeners();
  }

  setState(state) {
    this.currentState = state;
    document.body.className = `state-${state}`;
    console.log(`[UI State] -> ${state}`);

    // Update headers based on state
    const indicator = document.getElementById('hotkey-indicator');
    if (indicator) {
      if (state === 'listening') {
        indicator.textContent = 'Listening...';
        indicator.style.color = '#4CD964';
      } else if (state === 'processing') {
        indicator.textContent = 'Analyzing...';
        indicator.style.color = '#AF52DE';
      } else if (state === 'review') {
        indicator.textContent = 'Review';
        indicator.style.color = '#FFCC00';
      } else {
        indicator.textContent = '';
      }
    }

    if (state === 'idle') {
      this.tokens = [];
      this.selectedAlternatives = {};
      const altBox = document.getElementById('alternatives-box');
      if (altBox) altBox.classList.add('hidden');
    }
  }

  // Draw smooth, liquid Siri aurora gradients
  initCanvas() {
    const canvas = document.getElementById('sine-canvas');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');

    const renderAurora = () => {
      if (this.currentState !== 'listening') {
        requestAnimationFrame(renderAurora);
        return;
      }

      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Define 4 color blobs simulating the macOS Siri Aurora bar
      const time = this.phase;
      // Amplitude scales with microphone volume
      const amp = this.micLevel * 45.0;

      // Color 1: Siri Blue
      this.drawBlob(ctx, canvas, 
        canvas.width * 0.25 + Math.sin(time * 0.8) * 40, 
        canvas.height * 0.5 + Math.cos(time * 0.6) * 10, 
        60 + amp * 1.5, 
        'rgba(0, 122, 255, 0.7)'
      );

      // Color 2: Siri Purple
      this.drawBlob(ctx, canvas, 
        canvas.width * 0.5 + Math.cos(time * 0.9) * 35, 
        canvas.height * 0.5 + Math.sin(time * 0.7) * 8, 
        50 + amp * 1.8, 
        'rgba(175, 82, 222, 0.65)'
      );

      // Color 3: Siri Pink
      this.drawBlob(ctx, canvas, 
        canvas.width * 0.75 + Math.sin(time * 0.7) * 40, 
        canvas.height * 0.5 + Math.cos(time * 0.9) * 12, 
        55 + amp * 1.4, 
        'rgba(255, 45, 85, 0.6)'
      );

      // Color 4: Siri Teal (center blend helper)
      this.drawBlob(ctx, canvas, 
        canvas.width * 0.5 + Math.sin(time * 0.5) * 50, 
        canvas.height * 0.5 + Math.sin(time * 0.8) * 5, 
        45 + amp * 1.2, 
        'rgba(76, 217, 100, 0.4)'
      );

      this.phase += 0.04;
      requestAnimationFrame(renderAurora);
    };

    renderAurora();
  }

  // Draw fuzzy radial gradient particle helper representing liquid blobs
  drawBlob(ctx, canvas, x, y, radius, color) {
    ctx.save();
    ctx.globalCompositeOperation = 'screen';
    
    const grad = ctx.createRadialGradient(x, y, 0, x, y, radius);
    grad.addColorStop(0, color);
    grad.addColorStop(0.5, color.replace(/[\d\.]+\)$/, '0.2)'));
    grad.addColorStop(1, 'rgba(0, 0, 0, 0)');

    ctx.fillStyle = grad;
    ctx.beginPath();
    ctx.arc(x, y, radius, 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();
  }

  // Render review tokens with confidence levels
  renderTranscript(result) {
    const box = document.getElementById('transcript-display');
    const altBox = document.getElementById('alternatives-box');
    const altChips = document.getElementById('alt-chips');
    if (!box || !altBox || !altChips) return;

    box.innerHTML = '';
    altChips.innerHTML = '';
    this.tokens = result.terms || [];
    let hasLowConf = false;

    this.tokens.forEach((term, idx) => {
      const span = document.createElement('span');
      const termKey = `term_${idx}`;
      
      const displayWord = this.selectedAlternatives[termKey] || term.matched_name || term.original_word;
      span.textContent = displayWord + ' ';
      
      if (term.confidence_level === 'Medium') {
        span.className = 'term-medium';
      } else if (term.confidence_level === 'Low') {
        span.className = 'term-low';
        span.id = termKey;
        hasLowConf = true;

        if (term.alternatives && term.alternatives.length > 0) {
          term.alternatives.forEach(alt => {
            const btn = document.createElement('button');
            btn.className = 'chip-btn';
            btn.textContent = alt;
            btn.onclick = () => {
              this.selectedAlternatives[termKey] = alt;
              span.textContent = alt + ' ';
              span.className = '';
              btn.classList.add('selected');
              setTimeout(() => {
                altBox.classList.add('hidden');
              }, 300);
            };
            altChips.appendChild(btn);
          });
        }
      }
      box.appendChild(span);
    });

    if (hasLowConf) {
      altBox.classList.remove('hidden');
    } else {
      altBox.classList.add('hidden');
      setTimeout(() => {
        this.confirmAndInject();
      }, 700);
    }
  }

  confirmAndInject() {
    const box = document.getElementById('transcript-display');
    const finalText = box ? box.innerText.trim() : '';
    if (finalText && window.__TAURI__) {
      window.__TAURI__.invoke('confirm_and_inject', { text: finalText, strategy: 'ClipboardPaste' })
        .then(() => {
          this.setState('idle');
        })
        .catch(err => {
          console.error("Failed to inject:", err);
          this.setState('idle');
        });
    } else {
      this.setState('idle');
    }
  }

  // Tauri IPC events and commands listeners
  initTauriListeners() {
    if (!window.__TAURI__) {
      console.warn("Tauri wrapper not found. Running in browser mock mode.");
      return;
    }

    const { listen } = window.__TAURI__.event;

    // Listen to real-time microphone level updates
    listen('mic-level', (event) => {
      const targetLevel = parseFloat(event.payload) || 0.0;
      this.micLevel = this.micLevel * 0.35 + targetLevel * 0.65;
    });

    // Listen to state commands from global hotkeys
    listen('state-change', (event) => {
      const payload = event.payload;
      if (payload && payload.state) {
        this.setState(payload.state);
        
        if (payload.state === 'review' && payload.result) {
          this.renderTranscript(payload.result);
        }
      }
    });
  }

  initKeyboardListeners() {
    window.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        this.setState('idle');
        if (window.__TAURI__) {
          window.__TAURI__.invoke('cancel_dictation').catch(() => {});
        }
      } else if (e.key === 'Enter') {
        if (this.currentState === 'review') {
          this.confirmAndInject();
        }
      }
    });
  }
}

// Instantiate visualizer
window.addEventListener('DOMContentLoaded', () => {
  window.scribeUI = new ScribeRxUI();
});
