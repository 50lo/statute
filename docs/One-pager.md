# Statute: keeping product rules true

Oct 15, 2026 · @Anton Solo

Statute keeps a product's important rules written down, linked to the code, and checked every time the code changes.

## The problem

Every product runs on rules: "a cancelled order can't ship", "we keep a session even when it has no repository". Most of these rules live in people's heads, old tickets and chat threads.

When people leave, the rules fade. When AI writes the code, a rule can break quietly: the code compiles, the tests pass, and the product no longer does what we agreed. This gets more likely as more people contribute: PMs and designers now ship small features with coding agents too, without knowing every rule the code depends on.

## The idea

We write the important rules down in plain language. For each rule we record why it exists and who agreed to it. We link each rule to the code that makes it true and the tests. Test links identify checks of the rule’s behaviour. These checks can catch violations outside linked code. When the code changes, the system shows which rules are affected, and someone confirms they still hold.

If the code and a rule disagree, nobody quietly edits the rule to match the code. The disagreement is flagged and a person decides.

Statute answers three questions about the product:

1. **What must be true?**
2. **Why?**
3. **Is it still true?**

## Four concepts

| Concept | Plain meaning | Example |
| --- | --- | --- |
| **Terms** | The words we agree on, and what they are called in code | "Session" means one conversation between a developer and a coding agent. In code it is `Session`; the `sessions` table is something else (sign-in sessions) |
| **Rules** | What must be true | A session without a repository is still collected and kept |
| **Decisions** | Why a rule is the way it is, and who decided | We chose to keep repository-less sessions because a lot of real work happens outside repositories (decided by Anton) |
| **Links** | Where a rule lives in the code and tests | The rule above points to the collector's filter function and its test |

[Statute's glossary](Glossary.md) explains these concepts and the other words Statute uses, with examples.

Each rule states one observable obligation, with one case it protects and one nearby case that shows where it stops.

Every rule carries two statuses, because "we agreed on it" and "the code does it" are different facts:

- **Agreed?** proposed → agreed → retired
- **Built?** planned → built, or broken while a gap is open. A gap is a known mismatch between a rule and the code, tracked as an issue until a person resolves it.

Guides, such as a map of the code, sit outside Statute. **Guides follow the code; rules lead the code.** When code changes, a guide is updated to match. A rule is updated only by a new decision.

## How a rule stays true

Each built rule needs two kinds of links, and they do different jobs:

- **Code links** show where the rule is made true: the function, check or query that enforces it. When that code changes, the rule is flagged, and a person or agent confirms it still holds. Code links also tell a reader where to look and what is dangerous to touch.
- **Test links** point to tests that check the rule's behaviour. A test can catch a break even in code nobody linked, such as a new filter added somewhere else.

Neither is enough alone. Code links catch changes to the places we know about but miss new code. Tests catch new code but say nothing about why it matters or who decides. **A rule counts as built only when it has both.** The rare rule that no test can observe carries a manual check instead of a test.

Terms keep everyone using the same words. Each term records what a word means and what it is called in code, including code names that look alike but mean something else, such as the `sessions` table. A new term is approved like a rule, because the words in a term steer how agents name and structure code.

## The loop for every change

1. Before coding, the agent reads the rules and terms for the code it will touch.
2. After coding, the tool lists the rules the change affects.
3. The agent checks each affected rule. There are three outcomes:
   - **Holds**: the agent confirms it.
   - **Changes on purpose**: the agent proposes a decision, and a person approves it.
   - **Broken**: the agent opens a gap, and a person decides what to do.

Example: an agent adds a new filter that skips sessions without a repository. The filter is new code that no rule links to, but it makes the rule's linked test fail. The agent looks the test up in Statute, finds the rule "a session without a repository is still kept", checks it, finds it broken, and opens a gap instead of shipping. The review shows: *rule broken, gap open*. A person decides whether to fix the code or change the rule.

## Who does what

People hold the authority; the AI agent does the legwork; the tool does the mechanical checking.

| Who | Does | Never does |
| --- | --- | --- |
| **People** (PM, designer, engineer) | Agree rules and terms, approve decisions, resolve gaps | Keep the links up to date by hand |
| **AI agent** | Reads the rules and terms before coding, checks affected rules, proposes new rules and decisions, asks "what valid case would this rule reject?" | Turns its own proposal into an agreed rule, or edits a rule to match the code |
| **Tool** | Finds affected rules through links, checks that every built rule has code and test links, reports status | Judges whether a rule is right |

Anything an agent writes starts as *proposed*. It becomes binding only when a person approves it. A PM or designer shipping a small feature with an agent gets the same protection as an engineer: the agent reads the rules first, and the checks run on every change.

## What it is not, and how we start

Statute is small on purpose:

- **Not a full specification.** Only rules that are expensive to get wrong: money, privacy, permissions, what data we keep, what a metric means. Usually 10–30 per area.
- **Not paperwork on every change.** A change that touches no linked code and breaks no linked test adds nothing.
- **Not a guarantee.** Code, docs and tests can all agree and still be wrong. The cases people write for each rule are the safeguard.

We start with one area for 3–4 weeks, then count real problems caught, false alarms, and minutes spent per change. We expand only if the numbers justify it.
