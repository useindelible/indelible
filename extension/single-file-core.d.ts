declare module 'single-file-core/single-file.js' {
  export function getPageData(options: Record<string, unknown>): Promise<{ content: string }>
}
