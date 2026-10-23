import { UIAbility } from '@kit.AbilityKit';
import { window } from '@kit.ArkUI';
export default class EntryAbility extends UIAbility {
  onWindowStageCreate(stage: window.WindowStage): void {
    try {
      stage.loadContent('pages/Index');
    } catch (error) {
      console.error('Failed to load PhotoSelect page: ' + String(error));
    }
  }
}
