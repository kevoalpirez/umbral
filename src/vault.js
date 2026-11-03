const enc = new TextEncoder();
const dec = new TextDecoder();
export const b64 = bytes => { let s=''; for(const x of bytes) s+=String.fromCharCode(x); return btoa(s); };
export const unb64 = s => Uint8Array.from(atob(s), c=>c.charCodeAt(0));
export async function derive(password, salt) {
 const material = await crypto.subtle.importKey('raw',enc.encode(password),'PBKDF2',false,['deriveKey']);
 return crypto.subtle.deriveKey({name:'PBKDF2',salt,iterations:600000,hash:'SHA-256'},material,{name:'AES-GCM',length:256},false,['encrypt','decrypt']);
}
export function validateData(data) {
 if(data?.version!==1 || !Array.isArray(data.pages) || data.pages.length>10000) throw Error('Formato de espacio no válido.');
 const ids=new Set();
 for(const p of data.pages) {
  if(typeof p.id!=='string'||ids.has(p.id)||typeof p.title!=='string'||typeof p.body!=='string'||typeof p.deleted!=='boolean'||typeof p.favorite!=='boolean'||!Array.isArray(p.tags)||!p.tags.every(t=>typeof t==='string')||!(p.parent===null||typeof p.parent==='string')) throw Error('Página no válida.');
  if(p.cards!==undefined&&(!Array.isArray(p.cards)||!p.cards.every(c=>typeof c.id==='string'&&typeof c.front==='string'&&typeof c.back==='string'&&typeof c.known==='boolean')))throw Error('Tarjetas no válidas.');
  ids.add(p.id);
 }
 for(const p of data.pages){const seen=new Set([p.id]); let parent=p.parent; while(parent){if(seen.has(parent)||!ids.has(parent))throw Error('Jerarquía no válida.');seen.add(parent);parent=data.pages.find(x=>x.id===parent).parent;}}
 return data;
}
export async function seal(data,key,salt) {
 const iv=crypto.getRandomValues(new Uint8Array(12));
 const bytes=await crypto.subtle.encrypt({name:'AES-GCM',iv},key,enc.encode(JSON.stringify(validateData(data))));
 return JSON.stringify({format:'umbral-v1',salt:b64(salt),iv:b64(iv),ciphertext:b64(new Uint8Array(bytes))});
}
export async function open(raw,password) {
 const e=JSON.parse(raw);
 if(e.format!=='umbral-v1'||typeof e.ciphertext!=='string'||e.ciphertext.length>40000000)throw Error('Archivo no válido.');
 const salt=unb64(e.salt),iv=unb64(e.iv); if(salt.length!==16||iv.length!==12)throw Error('Archivo no válido.');
 const key=await derive(password,salt);
 const plain=await crypto.subtle.decrypt({name:'AES-GCM',iv},key,unb64(e.ciphertext));
 return {data:validateData(JSON.parse(dec.decode(plain))),key,salt};
}
