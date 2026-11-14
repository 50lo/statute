## Statute

- Before changing code, run `statute show` on the files you will touch and read the rules and terms it returns.
- Change a rule or an agreed term only together with a proposed decision that lists it (`statute decide`), never to match the code. Never edit `agreed` or `built`; the CLI sets them.
- When a user instruction conflicts with an agreed rule, name the rule and propose a decision before changing the code.
- After changing code, run the `change` skill before declaring the task done.
- When the task reveals a potential product rule, suggest it only if it is a distinct, observable product requirement supported by an explicit user instruction, product policy, or contract. Would the same requirement still have to hold after a major code refactor, a change to the user journey, and a complete UI redesign? If any answer is no or unclear, skip the suggestion. Check whether an existing rule already covers it. When suggesting a rule, name the consequence it prevents.
- Suggest a new term only when a stable domain concept needs a precise definition to apply a rule or interpret a product contract. Reuse an existing term if its meaning fits. A new code identifier or UI label alone is insufficient.
- Anything you create is `proposed`. Run `statute approve` only after a human in this session explicitly approves those specific decisions.
- Before asking a human to approve a rule, check it: what valid case would it reject? What invalid case would it allow? Does its meaning change with actor or permission, missing data, lifecycle stage, time, retries, concurrency or partial failure? Add to the rule only the answers that change it.
- Gaps go to the issue tracker named in this file; if none is named, use `statute gap open --note`.
- When instructions conflict: explicit user instruction > current task > project principles > agreed rules, terms and approved decisions > repo conventions > proposed decisions > agent judgment.
