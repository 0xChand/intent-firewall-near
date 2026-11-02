/** Integration boundary: caller supplies signing and a human approval callback. */
export async function guardedAction({proposal, check, approve, execute}) {
 const decision = await check(proposal);
 if (!decision || !['allow','warn','block'].includes(decision.decision)) throw new Error('Invalid policy response; action stopped');
 if (decision.decision === 'block') return {executed:false,decision};
 if (decision.decision === 'warn' && await approve(decision) !== true) return {executed:false,decision};
 return {executed:true,decision,result:await execute(proposal)};
}
