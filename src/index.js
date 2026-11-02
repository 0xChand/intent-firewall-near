export function evaluate(input, chain) {
 const reasons=[]; let decision='allow'; const elevate=(level, reason)=>{reasons.push(reason); if(({allow:0,warn:1,block:2})[level]>({allow:0,warn:1,block:2})[decision]) decision=level};
 const account=String(input.receiver||'').trim().toLowerCase(); const amount=Number(input.amount); const limit=Number(input.limit);
 if(!/^(?:[a-z0-9]+(?:[-_.][a-z0-9]+)*|[a-f0-9]{64})$/.test(account)||account.length<2||account.length>64) elevate('block','Invalid NEAR account ID.');
 if(!Number.isFinite(amount)||amount<=0||!Number.isFinite(limit)||limit<=0) elevate('block','Amount and limit must be positive numbers.');
 else if(amount>limit) elevate('block',`Amount exceeds the ${limit} NEAR per-action limit.`);
 if(input.blocked) elevate('block','Receiver appears on the local demo blocklist.');
 if(input.injection) elevate('block','Untrusted content attempted to override payment instructions.');
 if(chain?.status==='missing') elevate('block','Receiver does not exist on the selected NEAR network.');
 if(chain?.status==='error') elevate('warn','NEAR RPC could not verify the receiver; fail closed until a human reviews.');
 if(chain?.status==='found'&&!input.trusted) elevate('warn','First interaction with this receiver requires human approval.');
 if(!reasons.length) reasons.push('Receiver is trusted and amount is within the configured limit.');
 return {decision,reasons,receiver:account,amount,chain};
}
export async function checkAccount(account, network='mainnet', fetcher=fetch) {
 if(!/^[a-z0-9._-]{2,64}$/.test(account)) return {status:'invalid'};
 try {const response=await fetcher(network==='testnet'?'https://rpc.testnet.near.org':'https://rpc.mainnet.near.org',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({jsonrpc:'2.0',id:'intent-firewall',method:'query',params:{request_type:'view_account',finality:'final',account_id:account}}),signal:AbortSignal.timeout(8000)}); if(!response.ok) return {status:'error'}; const data=await response.json(); if(data.result) return {status:'found',network,codeHash:data.result.code_hash,blockHeight:data.result.block_height}; if(JSON.stringify(data.error).includes('UNKNOWN_ACCOUNT')) return {status:'missing',network}; return {status:'error',network};} catch{return {status:'error',network};}
}
export default {async fetch(request, env){const url=new URL(request.url); if(url.pathname==='/api/check'&&request.method==='POST'){try{const input=await request.json(); const chain=await checkAccount(String(input.receiver||'').trim().toLowerCase(),input.network==='testnet'?'testnet':'mainnet'); return Response.json(evaluate(input,chain),{headers:{'cache-control':'no-store'}})}catch{return Response.json({error:'Invalid request'}, {status:400})}} return env.ASSETS.fetch(request)}};
