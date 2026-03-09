/**
 * 历表二进制 fetch：优先未压缩 .bin（避免 dev 下 .br 的 Content-Encoding 导致 ERR_CONTENT_DECODING_FAILED），
 * .bin 404 时再请求 .br 并自行 Brotli 解压。
 */

/** 优先请求 url（.bin），200 则直接返回；404 时请求 url.br 并 Brotli 解压。支持带 ?v= 的 url。 */
export async function fetchBinaryMaybeBrotli(url: string): Promise<Uint8Array> {
  const qs = url.includes('?') ? url.slice(url.indexOf('?')) : ''
  const pathOnly = url.replace(/\?.*$/, '')
  const rBin = await fetch(url)
  if (rBin.ok) {
    return new Uint8Array(await rBin.arrayBuffer())
  }
  const rBr = await fetch(pathOnly + '.br' + qs)
  if (rBr.ok) {
    const ab = await rBr.arrayBuffer()
    const stream = new Response(ab).body!.pipeThrough(new DecompressionStream('brotli'))
    const reader = stream.getReader()
    const chunks: Uint8Array[] = []
    let len = 0
    while (true) {
      const { done, value } = await reader.read()
      if (done) break
      chunks.push(value)
      len += value.length
    }
    const out = new Uint8Array(len)
    let off = 0
    for (const c of chunks) {
      out.set(c, off)
      off += c.length
    }
    return out
  }
  throw new Error(`fetch ${url}: ${rBin.status}, .br: ${rBr.status}`)
}

export const VSOP87_MAGIC = new Uint8Array([0x56, 0x53, 0x42, 0x31]) // "VSB1"
export const ELP_MAGIC = new Uint8Array([0x45, 0x4c, 0x50, 0x31]) // "ELP1"

export function isVsop87Binary(buf: Uint8Array): boolean {
  if (buf.length < 4) return false
  return buf[0] === VSOP87_MAGIC[0] && buf[1] === VSOP87_MAGIC[1] && buf[2] === VSOP87_MAGIC[2] && buf[3] === VSOP87_MAGIC[3]
}

export function isElpBinary(buf: Uint8Array): boolean {
  if (buf.length < 4) return false
  return buf[0] === ELP_MAGIC[0] && buf[1] === ELP_MAGIC[1] && buf[2] === ELP_MAGIC[2] && buf[3] === ELP_MAGIC[3]
}
