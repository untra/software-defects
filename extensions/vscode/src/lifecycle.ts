export type ServerState = 'idle' | 'starting' | 'running' | 'stopping' | 'failed';

export class ServerLifecycle {
  state: ServerState = 'idle';
  private wanted = false;
  private generation = 0;
  private pending?: Promise<void>;
  private dirty = false;
  private failed = false;
  constructor(private readonly actions: {
    start(): Promise<void>;
    stop(): Promise<void>;
    invalidate(): void;
    changed(state: ServerState): void;
    error(error: unknown): void;
  }) {}

  update(wanted: boolean, retry = false): Promise<void> {
    if (wanted !== this.wanted || retry) {
      this.wanted = wanted;
      this.generation++;
      this.failed = false;
      this.actions.invalidate();
    }
    this.dirty = true;
    this.pending ??= Promise.resolve().then(() => this.reconcile()).finally(() => {
      this.pending = undefined;
      if (this.dirty) return this.update(this.wanted);
    });
    return this.pending;
  }

  crashed(): void {
    this.generation++;
    this.failed = true;
    this.actions.invalidate();
    this.setState('failed');
  }

  private setState(state: ServerState): void {
    this.state = state;
    this.actions.changed(state);
  }

  private async stop(): Promise<void> {
    this.setState('stopping');
    await this.actions.stop();
    this.setState('idle');
  }

  private async reconcile(): Promise<void> {
    while (this.dirty) {
      this.dirty = false;
      const generation = this.generation;
      try {
        if (!this.wanted) {
          if (this.state !== 'idle') await this.stop();
        } else if (this.state !== 'running' && !this.failed) {
          this.setState('starting');
          await this.actions.start();
          if (!this.wanted || generation !== this.generation) {
            await this.stop();
            this.dirty = true;
          } else this.setState('running');
        }
      } catch (error) {
        this.failed = generation === this.generation;
        this.actions.invalidate();
        try { await this.actions.stop(); } catch (stopError) { this.actions.error(stopError); }
        this.setState(this.failed ? 'failed' : 'idle');
        if (this.failed) this.actions.error(error);
        else this.dirty = true;
      }
    }
  }
}
