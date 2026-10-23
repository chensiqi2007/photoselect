declare module 'libphotoselect.so' {
  export function scanDirectory(path: string, recursive: boolean): Promise<string>;
  export function scanAuthorizedFiles(rootPath: string, fileList: string): Promise<string>;
}
