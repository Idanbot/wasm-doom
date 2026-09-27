/** One shared lane for spoken audio. Ambient chatter is dropped while busy;
 * important lines can wait for the current speaker to finish. */
export class VoiceGate {
  private active = false;
  private pending: ((done: () => void) => void) | null = null;

  get busy() {
    return this.active || this.pending !== null;
  }

  tryStart(start: (done: () => void) => void): boolean {
    if (this.busy) return false;
    this.active = true;
    let finished = false;
    const done = () => {
      if (finished) return;
      finished = true;
      this.active = false;
      const next = this.pending;
      this.pending = null;
      if (next) this.tryStart(next);
    };
    try {
      start(done);
    } catch (error) {
      done();
      throw error;
    }
    return true;
  }

  enqueue(start: (done: () => void) => void) {
    if (!this.tryStart(start)) this.pending = start;
  }

  clearPending() {
    this.pending = null;
  }
}
