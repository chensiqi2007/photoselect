import picker from '@ohos.file.picker';
import fs from '@ohos.file.fs';
import { scanAuthorizedFiles, scanDirectory } from 'libphotoselect.so';
interface PhotoGroup {
  id: string; base_name: string; folder_path: string; std_image: string | null; raw_image: string | null;
  xmp_sidecar: string | null; rating: number; flag: number; color_label: string; file_size_bytes: number;
}
interface ScanSummary {
  folder_path: string; total_groups: number; raw_plus_jpg_count: number; raw_only_count: number;
  std_only_count: number; scan_elapsed_ms: number; groups: PhotoGroup[];
}
interface ScanResponse { data?: ScanSummary; error?: string; }
@Entry
@Component
struct Index {
  @State pathInput: string = '';
  @State recursive: boolean = false;
  @State photos: PhotoGroup[] = [];
  @State statusText: string = '输入已获系统授权的照片目录，然后开始扫描';
  @State busy: boolean = false;
  @State filter: string = '全部';
  @State selectedUri: string = '';

  private async chooseFolder(): Promise<void> {
    try {
      const documentPicker = new picker.DocumentViewPicker();
      const options = {
        maxSelectNumber: 1,
        selectMode: picker.DocumentSelectMode.FOLDER,
      } as picker.DocumentSelectOptions;
      const uris: string[] = await documentPicker.select(options);
      if (uris.length === 0) {
        this.statusText = '已取消目录选择';
        return;
      }
      this.selectedUri = uris[0];
        this.pathInput = this.selectedUri;
      this.statusText = '已获得目录授权，可以开始扫描';
    } catch (error) {
      this.statusText = '目录授权失败：' + String(error);
    }
  }

  private joinAuthorizedUri(rootUri: string, listedPath: string): string {
    if (listedPath.startsWith('file://') || listedPath.startsWith('datashare://') || listedPath.startsWith('dataability://')) {
      return listedPath;
    }
    const root = rootUri.endsWith('/') ? rootUri.substring(0, rootUri.length - 1) : rootUri;
    const child = listedPath.startsWith('/') ? listedPath : '/' + listedPath;
    return root + child;
  }

  private async scanAuthorizedFolder(): Promise<string> {
    try {
      const listed: string[] = await fs.listFile(this.selectedUri, { recursion: this.recursive });
      const entries: Array<[string, number]> = [];
      for (let index = 0; index < listed.length; index++) {
        const uri = this.joinAuthorizedUri(this.selectedUri, listed[index]);
        const lowerName = listed[index].toLowerCase();
        if (lowerName.endsWith('.jpg') || lowerName.endsWith('.jpeg') || lowerName.endsWith('.png') ||
          lowerName.endsWith('.heic') || lowerName.endsWith('.heif') || lowerName.endsWith('.webp') ||
          lowerName.endsWith('.tif') || lowerName.endsWith('.tiff') || lowerName.endsWith('.arw') ||
          lowerName.endsWith('.cr2') || lowerName.endsWith('.cr3') || lowerName.endsWith('.nef') ||
          lowerName.endsWith('.nrw') || lowerName.endsWith('.raf') || lowerName.endsWith('.rw2') ||
          lowerName.endsWith('.orf') || lowerName.endsWith('.pef') || lowerName.endsWith('.dng') ||
          lowerName.endsWith('.3fr') || lowerName.endsWith('.iiq') || lowerName.endsWith('.xmp')) {
          entries.push([uri, 0]);
        }
      }
      return await scanAuthorizedFiles(this.selectedUri, JSON.stringify(entries));
    } catch (error) {
      return JSON.stringify({ error: '无法枚举已授权目录：' + String(error) });
    }
  }
  private async runScan(): Promise<void> {
    const path = this.pathInput.trim();
    if (path.length === 0) { this.statusText = '请先输入照片目录路径'; return; }
    this.busy = true; this.statusText = '正在调用 Rust 扫描核心…';
    try {
      const payload = this.selectedUri.length > 0
        ? await this.scanAuthorizedFolder()
        : await scanDirectory(path, this.recursive);
      const response = JSON.parse(payload) as ScanResponse;
      if (response.error !== undefined) throw new Error(response.error);
      if (response.data === undefined) throw new Error('扫描器没有返回目录数据');
      this.photos = response.data.groups;
      this.statusText = '已扫描 ' + response.data.total_groups + ' 组 · RAW+JPG ' + response.data.raw_plus_jpg_count + ' · ' + response.data.scan_elapsed_ms + ' ms';
    } catch (error) { this.statusText = '扫描失败：' + String(error); }
    finally { this.busy = false; }
  }
  private visiblePhotos(): PhotoGroup[] {
    if (this.filter === '全部') return this.photos;
    const result: PhotoGroup[] = [];
    for (let i = 0; i < this.photos.length; i++) {
      const p = this.photos[i];
      if (this.filter === 'RAW+JPG' && p.std_image !== null && p.raw_image !== null) result.push(p);
      else if (this.filter === '留用' && p.flag === 1) result.push(p);
      else if (this.filter === '淘汰' && p.flag === -1) result.push(p);
    }
    return result;
  }
  private imagePath(photo: PhotoGroup): string { return photo.std_image !== null ? photo.std_image : (photo.raw_image !== null ? photo.raw_image : ''); }
  private markRating(id: string, rating: number): void {
    for (let i = 0; i < this.photos.length; i++) if (this.photos[i].id === id) { this.photos[i].rating = rating; this.photos = this.photos.slice(); return; }
  }
  private markFlag(id: string, flag: number): void {
    for (let i = 0; i < this.photos.length; i++) if (this.photos[i].id === id) { this.photos[i].flag = flag; this.photos = this.photos.slice(); return; }
  }
  private badge(photo: PhotoGroup): string {
    if (photo.std_image !== null && photo.raw_image !== null) return 'RAW+JPG';
    return photo.raw_image !== null ? 'RAW' : 'JPG';
  }
  private flagText(flag: number): string { return flag === 1 ? '留用' : (flag === -1 ? '淘汰' : '未标记'); }
  build() {
    Column({ space: 14 }) {
      Row() {
        Column({ space: 4 }) {
          Text('PhotoSelect').fontSize(25).fontWeight(FontWeight.Bold).fontColor('#F5F7FB');
          Text('HarmonyOS PC / 2-in-1').fontSize(12).fontColor('#8C97AA');
        }.alignItems(HorizontalAlign.Start).layoutWeight(1);
        Button(this.busy ? '扫描中…' : '开始扫描').enabled(!this.busy).backgroundColor('#38BDF8').fontColor('#07111D').onClick(() => this.runScan());
      }.width('100%');
      Row({ space: 10 }) {
        TextInput({ text: this.pathInput, placeholder: '照片目录，例如 D:/Photos 或已授权路径' })
          .layoutWeight(1).height(44).fontColor('#F5F7FB').placeholderColor('#7B879B').backgroundColor('#1A202D').borderRadius(8)
          .onChange((value: string) => {
            this.pathInput = value;
            if (value !== this.selectedUri) this.selectedUri = '';
          });
        Button('选择目录').height(44).backgroundColor('#263247').fontColor('#D8E2F0').onClick(() => this.chooseFolder());
        Button(this.recursive ? '递归：开' : '递归：关').height(44).backgroundColor('#263247').fontColor('#D8E2F0').onClick(() => this.recursive = !this.recursive);
      }.width('100%');
      Row({ space: 8 }) {
        Text(this.statusText).fontSize(13).fontColor('#AEB9CA').layoutWeight(1);
        Select([{ value: '全部' }, { value: 'RAW+JPG' }, { value: '留用' }, { value: '淘汰' }])
          .value(this.filter).height(34).onSelect((index: number) => { const values = ['全部', 'RAW+JPG', '留用', '淘汰']; this.filter = values[index]; });
      }.width('100%');
      if (this.photos.length === 0) {
        Column({ space: 10 }) {
          Text('还没有照片').fontSize(21).fontColor('#DDE7F5');
          Text('Rust 负责扫描与 JPG/RAW 同名配对，照片源目录保持只读。').fontSize(13).fontColor('#8C97AA');
          Text('请先确认系统已授权所选目录。').fontSize(12).fontColor('#6F7C90');
        }.justifyContent(FlexAlign.Center).layoutWeight(1).width('100%');
      } else {
        Grid() {
          ForEach(this.visiblePhotos(), (photo: PhotoGroup) => {
            GridItem() {
              Column({ space: 8 }) {
                if (this.imagePath(photo).length > 0) {
                  Image(this.imagePath(photo)).width('100%').height(170).objectFit(ImageFit.Cover).borderRadius(8);
                } else {
                  Text('无预览').width('100%').height(170).textAlign(TextAlign.Center).fontColor('#8895A8').backgroundColor('#202735').borderRadius(8);
                }
                Row() {
                  Text(photo.base_name).fontSize(14).fontColor('#EAF0F8').maxLines(1).layoutWeight(1);
                  Text(this.badge(photo)).fontSize(10).fontColor('#38BDF8');
                }
                Row({ space: 6 }) {
                  Button('★ ' + photo.rating).height(30).fontSize(12).backgroundColor('#263247').fontColor('#F5C451')
                    .onClick(() => this.markRating(photo.id, photo.rating >= 5 ? 0 : photo.rating + 1));
                  Button(this.flagText(photo.flag)).height(30).fontSize(12)
                    .backgroundColor(photo.flag === 1 ? '#164E3A' : (photo.flag === -1 ? '#4A2630' : '#263247'))
                    .fontColor('#EAF0F8').onClick(() => this.markFlag(photo.id, photo.flag === 0 ? 1 : (photo.flag === 1 ? -1 : 0)));
                }.width('100%');
              }.padding(10).backgroundColor('#171D29').borderRadius(12);
            }
          }, (photo: PhotoGroup) => photo.id);
        }.columnsTemplate('1fr 1fr 1fr 1fr').columnsGap(12).rowsGap(12).layoutWeight(1).width('100%');
      }
    }.padding({ top: 22, bottom: 22, left: 28, right: 28 }).width('100%').height('100%').backgroundColor('#101116');
  }
}
