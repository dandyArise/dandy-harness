// Explicit diagnostic mode only; not loaded in normal launches.
import { Session } from 'node:inspector';
import { writeFileSync } from 'node:fs';
const session = new Session();
session.connect();
session.post('Profiler.enable', () => session.post('Profiler.start'));
setTimeout(() => {
  session.post('Profiler.stop', (error, result) => {
    if (!error) writeFileSync(process.env.DANDY_NODE_PROFILE, JSON.stringify(result.profile));
    session.disconnect();
  });
}, 4500).unref();
