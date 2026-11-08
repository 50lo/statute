import { access } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

const LINK_FINDING_CODES = new Set(["link.stale", "link.missing"]);
const CHECK_TIMEOUT_MS = 30_000;

interface CheckReport {
  findings?: Array<{ code?: unknown }>;
}

async function findStatuteRoot(cwd: string): Promise<string | undefined> {
  let directory = resolve(cwd);

  while (true) {
    try {
      await access(join(directory, "statute.yml"));
      return directory;
    } catch {
      const parent = dirname(directory);
      if (parent === directory) return undefined;
      directory = parent;
    }
  }
}

function hasStaleOrMissingLinks(stdout: string): boolean {
  try {
    const report = JSON.parse(stdout) as CheckReport;
    return report.findings?.some(
      (finding) => typeof finding.code === "string" && LINK_FINDING_CODES.has(finding.code),
    ) ?? false;
  } catch {
    return false;
  }
}

export default function (pi: ExtensionAPI) {
  // Ask once for each incoming user prompt, even if the reminder's continuation
  // still leaves stale or missing links.
  let reminderRequested = false;

  pi.on("input", (event) => {
    if (event.source !== "extension") reminderRequested = false;
  });

  pi.on("agent_before_settle", async (event, ctx) => {
    if (reminderRequested) return;

    const root = await findStatuteRoot(ctx.cwd);
    if (!root) return;

    let result;
    try {
      result = await pi.exec("statute", ["check", "--json"], {
        cwd: root,
        timeout: CHECK_TIMEOUT_MS,
      });
    } catch {
      // Statute is an optional CLI dependency for this integration.
      return;
    }

    // `statute check` can exit non-zero when findings exist; its JSON report is
    // still the source of truth for deciding whether this reminder applies.
    if (!hasStaleOrMissingLinks(result.stdout)) return;

    reminderRequested = true;
    return {
      entries: [
        ...event.entries,
        {
          type: "custom_message",
          customType: "statute-change-reminder",
          content:
            "Statute check found stale or missing links. Before finishing this task, read and follow the Statute `change` skill, review each affected rule, record its verdict, and run the check again. Do not treat this reminder as a review verdict.",
          display: false,
        },
      ],
      continue: true,
    };
  });
}
