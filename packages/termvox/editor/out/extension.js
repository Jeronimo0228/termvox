"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const node_child_process_1 = require("node:child_process");
const node_util_1 = require("node:util");
const vscode = __importStar(require("vscode"));
const execFileAsync = (0, node_util_1.promisify)(node_child_process_1.execFile);
function resolveCliPath(configured) {
    const trimmed = configured.trim();
    if (trimmed.length === 0 || trimmed === "termvox") {
        return "termvox";
    }
    if (/[\0;&|`$<>]/.test(trimmed)) {
        throw new Error("termvox.cliPath contains unsupported characters");
    }
    return trimmed;
}
function activate(context) {
    const status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    status.command = "termvox.talk";
    status.tooltip = "TermVox: push-to-talk";
    const refreshStatus = async () => {
        if (!vscode.workspace.getConfiguration("termvox").get("showStatusBar", true)) {
            status.hide();
            return;
        }
        const running = await daemonRunning();
        status.text = running ? "$(mic-filled) TermVox" : "$(mic) TermVox";
        status.show();
    };
    context.subscriptions.push(status, vscode.commands.registerCommand("termvox.talk", () => runTermvox(["talk"])), vscode.commands.registerCommand("termvox.daemonStart", () => runTermvox(["daemon", "start", "--background"])), vscode.commands.registerCommand("termvox.daemonStop", () => runTermvox(["daemon", "stop"])), vscode.commands.registerCommand("termvox.daemonStatus", () => runTermvox(["daemon", "status"])), vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration("termvox")) {
            void refreshStatus();
        }
    }));
    void refreshStatus();
    const timer = setInterval(() => void refreshStatus(), 15_000);
    context.subscriptions.push({ dispose: () => clearInterval(timer) });
}
async function runTermvox(args) {
    const cliPath = resolveCliPath(vscode.workspace.getConfiguration("termvox").get("cliPath", "termvox"));
    try {
        const { stdout, stderr } = await execFileAsync(cliPath, args, {
            env: process.env,
            timeout: 120_000,
        });
        const output = [stdout, stderr].filter(Boolean).join("\n").trim();
        if (output) {
            vscode.window.showInformationMessage(output.slice(0, 500));
        }
    }
    catch (error) {
        const message = error instanceof Error ? error.message : "TermVox command failed";
        vscode.window.showErrorMessage(message);
    }
}
async function daemonRunning() {
    const cliPath = resolveCliPath(vscode.workspace.getConfiguration("termvox").get("cliPath", "termvox"));
    try {
        const { stdout } = await execFileAsync(cliPath, ["daemon", "status"], {
            env: process.env,
            timeout: 5_000,
        });
        return /running/i.test(stdout);
    }
    catch {
        return false;
    }
}
function deactivate() { }
