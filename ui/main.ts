interface MatchedTerm {
  word: string;
  confidence: 'high' | 'medium' | 'low';
  alternatives?: string[];
}

class ScribeRxUI {
  private currentState: 'idle' | 'listening' | 'processing' | 'review' = 'idle';

  constructor() {
    this.initKeyboardListeners();
  }

  public setState(state: 'idle' | 'listening' | 'processing' | 'review') {
    this.currentState = state;
    document.body.className = `state-${state}`;
  }

  public renderTranscript(terms: MatchedTerm[]) {
    const box = document.getElementById('transcript-display');
    const altBox = document.getElementById('alternatives-box');
    const altChips = document.getElementById('alt-chips');
    if (!box || !altBox || !altChips) return;

    box.innerHTML = '';
    altChips.innerHTML = '';
    let hasLowConf = false;

    terms.forEach(term => {
      const span = document.createElement('span');
      span.textContent = term.word + ' ';
      
      if (term.confidence === 'medium') {
        span.className = 'term-medium';
      } else if (term.confidence === 'low') {
        span.className = 'term-low';
        hasLowConf = true;

        if (term.alternatives) {
          term.alternatives.forEach(alt => {
            const btn = document.createElement('button');
            btn.className = 'chip-btn';
            btn.textContent = alt;
            btn.onclick = () => {
              span.textContent = alt + ' ';
              span.className = '';
              altBox.classList.add('hidden');
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
    }
  }

  private initKeyboardListeners() {
    window.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        this.setState('idle');
      } else if (e.key === 'Enter' && this.currentState === 'review') {
        this.setState('idle');
      }
    });
  }
}

// Global UI instance for Tauri window IPC
const scribeUI = new ScribeRxUI();
(window as any).scribeUI = scribeUI;
