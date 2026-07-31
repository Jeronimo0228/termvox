import { execFile } from "node:child_process";
import { promisify } from "node:util";
import * as vscode from "vscode";

const execFileAsync = promisify(execFile);

function resolveCliPath(configured: string): string {
  const trimmed = configured.trim();
  if (trimmed.length === 0 || trimmed === "termvox") {
    return "termvox";
  }
  if (/[\0;&|`$<>]/.test(trimmed)) {
    throw new Error("termvox.cliPath contains unsupported characters");
  }
  return trimmed;
}

export function activate(context: vscode.ExtensionContext): void {
  const status = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    100,
  );
  status.command = "termvox.talk";
  status.tooltip = "TermVox: push-to-talk";

  const refreshStatus = async (): Promise<void> => {
    if (!vscode.workspace.getConfiguration("termvox").get<boolean>("showStatusBar", true)) {
      status.hide();
      return;
    }
    const running = await daemonRunning();
    status.text = running ? "$(mic-filled) TermVox" : "$(mic) TermVox";
    status.show();
  };

  context.subscriptions.push(
    status,
    vscode.commands.registerCommand("termvox.talk", () => runTermvox(["talk"])),
    vscode.commands.registerCommand("termvox.daemonStart", () =>
      runTermvox(["daemon", "start", "--background"]),
    ),
    vscode.commands.registerCommand("termvox.daemonStop", () =>
      runTermvox(["daemon", "stop"]),
    ),
    vscode.commands.registerCommand("termvox.daemonStatus", () =>
      runTermvox(["daemon", "status"]),
    ),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("termvox")) {
        void refreshStatus();
      }
    }),
  );

  void refreshStatus();
  const timer = setInterval(() => void refreshStatus(), 15_000);
  context.subscriptions.push({ dispose: () => clearInterval(timer) });
}

async function runTermvox(args: string[]): Promise<void> {
  const cliPath = resolveCliPath(
    vscode.workspace.getConfiguration("termvox").get<string>("cliPath", "termvox"),
  );
  try {
    const { stdout, stderr } = await execFileAsync(cliPath, args, {
      env: process.env,
      timeout: 120_000,
    });
    const output = [stdout, stderr].filter(Boolean).join("\n").trim();
    if (output) {
      vscode.window.showInformationMessage(output.slice(0, 500));
    }
  } catch (error) {
    const message =
      error instanceof Error ? error.message : "TermVox command failed";
    vscode.window.showErrorMessage(message);
  }
}

async function daemonRunning(): Promise<boolean> {
  const cliPath = resolveCliPath(
    vscode.workspace.getConfiguration("termvox").get<string>("cliPath", "termvox"),
  );
  try {
    const { stdout } = await execFileAsync(cliPath, ["daemon", "status"], {
      env: process.env,
      timeout: 5_000,
    });
    return /running/i.test(stdout);
  } catch {
    return false;
  }
}

export function deactivate(): void {}
