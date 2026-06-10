---
name: requirements-definition
description: >-
  Turns user requests into structured requirements documents as standalone HTML
  files. Clarifies ambiguities with targeted questions before writing. Use when
  the user asks for a new feature, change request, 要件定義, spec, requirements,
  or before starting implementation of unclear requirements.
---

# Requirements Definition

## When To Use

Use this skill when the user asks to define requirements, prepare a spec, shape a feature request, or start implementation from an unclear request.

Do not use this skill for simple bug fixes, small copy edits, or direct questions where a requirements document would add unnecessary overhead.

## Core Workflow

1. Analyze the user's request.
2. Ask targeted clarification questions before writing when requirements are ambiguous.
3. Generate a standalone HTML requirements document from `template.html`.
4. Save it to `docs/requirements/{feature-slug}.html`.
5. Summarize the document in chat and ask for approval.
6. Do not start implementation until the user approves the requirements.

## Phase 1: Analyze The Request

Extract these points from the user's request:

- Target: feature, screen, bug, workflow, or system being changed.
- Action: create, update, remove, fix, or document.
- Users: who uses the feature.
- Success condition: what must be true when the work is done.
- Constraints: existing behavior, technical limits, deadlines, and explicit exclusions.

Keep this analysis concise. Share it only when it helps the user confirm your understanding.

## Phase 2: Clarify First

If important information is missing, ask questions before writing the HTML document.

When the `AskQuestion` tool is available, use it and ask only 1-2 critical questions at a time. Prioritize questions in this order:

1. Scope boundaries: what is included and excluded.
2. Acceptance criteria: how completion will be verified.
3. User flow and UI behavior.
4. Non-functional requirements such as performance, security, or availability.

Continue clarifying until the requirements are specific enough to produce a useful document.

## Phase 3: Generate HTML

Use `template.html` as the base. Replace placeholders with complete Japanese content.

Required sections:

- Metadata: title, date, status, and target.
- Background and overview.
- Goals.
- Scope: include both in-scope and out-of-scope items.
- User scenarios.
- Functional requirements with numbered Must/Should items.
- Acceptance criteria that are testable.
- Constraints and assumptions.
- Open questions, even if the section says none.

Optional sections:

- Non-functional requirements.
- UI/UX requirements.
- Data requirements.
- Glossary.

If an optional section does not apply, keep the section and write a short note such as `<p class="empty-note">該当なし。</p>`.

## HTML Output Rules

- Save generated requirements to `docs/requirements/{feature-slug}.html`.
- Use kebab-case ASCII slugs, for example `alliance-donation.html`.
- Keep the HTML self-contained with embedded CSS only.
- Do not add external scripts, fonts, images, or network dependencies.
- Write body text in Japanese unless the user explicitly requests another language.
- Use semantic HTML headings, lists, and tables rather than dumping plain text into one block.
- Escape user-provided text where needed so it does not break the HTML.

## Chat Summary

After saving the HTML file, respond with:

- The path to the generated requirements file.
- A short summary of the goal and scope.
- The top acceptance criteria.
- Any remaining open questions.
- A clear request for approval before implementation.

Keep the chat response brief. The HTML file is the source of detail.

## Quality Checklist

Before finishing, verify:

- The document clearly separates in-scope and out-of-scope work.
- Each functional requirement is observable or testable.
- Acceptance criteria can be verified without guessing intent.
- Assumptions and open questions are explicit.
- The generated HTML opens standalone in a browser.
- Implementation has not started before approval.

## Additional Resources

- Use [template.html](template.html) as the HTML document template.
