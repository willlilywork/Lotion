import {
  MessageChannelMain,
  utilityProcess,
  type UtilityProcess,
  type WebContents,
} from 'electron';
import { IPC_DB_PORT } from '../shared/ipc.js';

/** Starts the DB utilityProcess and hands each renderer a direct MessagePort to it. */
export class DbBroker {
  private child: UtilityProcess | undefined;

  constructor(private readonly hostPath: string) {}

  start(): void {
    this.child = utilityProcess.fork(this.hostPath, [], {
      serviceName: 'Lotion Database',
      stdio: 'inherit',
    });
    this.child.on('exit', (code) => {
      console.error(`db process exited with code ${String(code)}`);
      this.child = undefined;
    });
  }

  get pid(): number | undefined {
    return this.child?.pid;
  }

  connect(contents: WebContents): void {
    if (!this.child) throw new Error('db process not running');
    const { port1, port2 } = new MessageChannelMain();
    this.child.postMessage({ type: 'connect' }, [port1]);
    contents.postMessage(IPC_DB_PORT, null, [port2]);
  }

  stop(): void {
    this.child?.kill();
  }
}
