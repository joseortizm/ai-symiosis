import { vi } from 'vitest';

export const mockInvoke = vi.fn();

export const mockSearchManager = {
  searchImmediate: vi.fn().mockResolvedValue([]),
  updateState: vi.fn(),
};

export const mockDialogManager = {
  closeCreateDialog: vi.fn(),
  closeDeleteDialog: vi.fn(),
  closeRenameDialog: vi.fn(),
  newNoteName: 'test-note',
  newNoteNameForRename: 'renamed-note',
};

export const resetAllMocks = () => {
  vi.clearAllMocks();
  mockInvoke.mockClear();
  mockSearchManager.searchImmediate.mockResolvedValue([]);
};