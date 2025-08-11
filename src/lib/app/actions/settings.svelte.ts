interface SettingsActionDeps {
  configService: {
    openPane: () => Promise<void>;
    closePane: () => void;
  };
  focusManager: {
    focusSearch: () => void;
  };
}

export function createSettingsActions(deps: SettingsActionDeps) {
  const { configService, focusManager } = deps;

  async function openSettingsPane(): Promise<void> {
    await configService.openPane();
    focusManager.focusSearch();
  }

  function closeSettingsPane(): void {
    configService.closePane();
    focusManager.focusSearch();
  }

  return {
    openSettingsPane,
    closeSettingsPane
  };
}
