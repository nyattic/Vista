interface Toast {
  id: number;
  message: string;
  tone: 'info' | 'success' | 'warning' | 'danger';
}

interface ConfirmState {
  title: string;
  message: string;
  confirmLabel: string;
  tone: 'normal' | 'danger';
  resolve: (value: boolean) => void;
}

class UiStore {
  toasts = $state<Toast[]>([]);
  confirmState = $state<ConfirmState | null>(null);
  private seq = 0;

  toast(message: string, tone: Toast['tone'] = 'info') {
    const id = ++this.seq;
    this.toasts = [...this.toasts, { id, message, tone }];
    window.setTimeout(() => this.dismissToast(id), 2600);
  }

  dismissToast(id: number) {
    this.toasts = this.toasts.filter((t) => t.id !== id);
  }

  confirm(options: {
    title: string;
    message: string;
    confirmLabel?: string;
    tone?: ConfirmState['tone'];
  }): Promise<boolean> {
    return new Promise((resolve) => {
      this.confirmState = {
        title: options.title,
        message: options.message,
        confirmLabel: options.confirmLabel ?? 'Confirm',
        tone: options.tone ?? 'normal',
        resolve
      };
    });
  }

  answerConfirm(value: boolean) {
    const state = this.confirmState;
    if (!state) return;
    this.confirmState = null;
    state.resolve(value);
  }
}

export const uiStore = new UiStore();
