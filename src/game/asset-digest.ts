/** Public-file checksum fallback for local LAN HTTP, where SubtleCrypto is absent. */
const primes = [2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59,61,67,71,73,79,83,89,97,101,103,107,109,113,127,131,137,139,149,151,157,163,167,173,179,181,191,193,197,199,211,223,227,229,233,239,241,251,257,263,269,271,277,281,283,293,307,311];
const roundConstants = primes.map(p => Math.floor((Math.cbrt(p)%1)*0x100000000));
const rotate = (x: number, n: number) => (x >>> n) | (x << (32-n));

export function softwareAssetDigest(bytes: ArrayBuffer): string {
  const size=bytes.byteLength, padded=new Uint8Array(Math.ceil((size+9)/64)*64);
  padded.set(new Uint8Array(bytes)); padded[size]=0x80;
  const view=new DataView(padded.buffer);
  view.setUint32(padded.length-8,Math.floor(size/0x20000000)); view.setUint32(padded.length-4,size*8);
  const hash=primes.slice(0,8).map(p=>Math.floor((Math.sqrt(p)%1)*0x100000000));
  const words=new Uint32Array(64);
  for(let offset=0;offset<padded.length;offset+=64) {
    for(let i=0;i<16;i++)words[i]=view.getUint32(offset+i*4);
    for(let i=16;i<64;i++) {
      const x=words[i-15]!,y=words[i-2]!;
      words[i]=(rotate(x,7)^rotate(x,18)^(x>>>3))+words[i-16]!+(rotate(y,17)^rotate(y,19)^(y>>>10))+words[i-7]!;
    }
    let [a,b,c,d,e,f,g,h]=hash as [number,number,number,number,number,number,number,number];
    for(let i=0;i<64;i++) {
      const t1=(h+(rotate(e,6)^rotate(e,11)^rotate(e,25))+((e&f)^(~e&g))+roundConstants[i]!+words[i]!)|0;
      const t2=((rotate(a,2)^rotate(a,13)^rotate(a,22))+((a&b)^(a&c)^(b&c)))|0;
      h=g;g=f;f=e;e=(d+t1)|0;d=c;c=b;b=a;a=(t1+t2)|0;
    }
    const result=[a,b,c,d,e,f,g,h];for(let i=0;i<8;i++)hash[i]=(hash[i]!+result[i]!)>>>0;
  }
  return hash.map(x=>x.toString(16).padStart(8,"0")).join("");
}

export async function assetDigest(bytes: ArrayBuffer) {
  if(!globalThis.crypto?.subtle)return softwareAssetDigest(bytes);
  return Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256",bytes)),b=>b.toString(16).padStart(2,"0")).join("");
}
