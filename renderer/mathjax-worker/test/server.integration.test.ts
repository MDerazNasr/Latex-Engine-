import assert from "node:assert/strict";
import { once } from "node:events";
import { fileURLToPath } from "node:url";
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
import test from "node:test";

test("stdio server handshakes and renders one correlated request", async () => {
  const serverPath = fileURLToPath(
    new URL("../src/server.js", import.meta.url),
  );
  const child = spawn(process.execPath, [serverPath], {
    stdio: ["pipe", "pipe", "pipe"],
  });
  const stderr: Buffer[] = [];
  child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
  const lines = createInterface({
    input: child.stdout,
    crlfDelay: Number.POSITIVE_INFINITY,
  });
  const iterator = lines[Symbol.asyncIterator]();

  try {
    const ready = JSON.parse(await nextLine(iterator)) as {
      type?: string;
      protocol?: number;
    };
    assert.equal(ready.protocol, 1);
    assert.equal(ready.type, "ready");

    child.stdin.write(
      `${JSON.stringify({
        protocol: 1,
        id: "integration-1",
        method: "render",
        params: { source: "E=mc^2", displayMode: true },
      })}\n`,
    );
    const response = JSON.parse(await nextLine(iterator)) as {
      id?: string;
      ok?: boolean;
      result?: { svgUtf8?: string };
    };
    assert.equal(response.id, "integration-1");
    assert.equal(response.ok, true);
    assert.match(response.result?.svgUtf8 ?? "", /^<svg /);

    const exited = once(child, "exit");
    child.stdin.end();
    const [exitCode] = await exited;
    assert.equal(exitCode, 0);
    assert.equal(Buffer.concat(stderr).toString("utf8"), "");
  } finally {
    lines.close();
    if (child.exitCode === null) {
      child.kill();
    }
  }
});

async function nextLine(iterator: AsyncIterator<string>): Promise<string> {
  let timeout: NodeJS.Timeout | undefined;
  try {
    const result = await Promise.race([
      iterator.next(),
      new Promise<never>((_resolve, reject) => {
        timeout = setTimeout(
          () => reject(new Error("worker response timed out")),
          5_000,
        );
      }),
    ]);
    assert.equal(result.done, false);
    return result.value;
  } finally {
    if (timeout !== undefined) {
      clearTimeout(timeout);
    }
  }
}
