import { ConfigurationChangeEvent, workspace } from 'vscode';
import { Config } from './Config';
import { IDisposable } from './types';

export class ConfigService implements IDisposable {
  private static readonly _namespace = 'oxc';
  private readonly _disposables: IDisposable[] = [];

  public config: Config;

  public onConfigChange:
    | ((this: ConfigService, config: ConfigurationChangeEvent) => Promise<void>)
    | undefined;

  constructor() {
    this.config = new Config();
    this.onConfigChange = undefined;

    const disposeChangeListener = workspace.onDidChangeConfiguration(
      this.onVscodeConfigChange.bind(this),
    );
    this._disposables.push(disposeChangeListener);
  }

  private async onVscodeConfigChange(event: ConfigurationChangeEvent): Promise<void> {
    if (event.affectsConfiguration(ConfigService._namespace)) {
      this.config.refresh();
      await this.onConfigChange?.(event);
    }
  }

  dispose() {
    for (const disposable of this._disposables) {
      disposable.dispose();
    }
  }
}
