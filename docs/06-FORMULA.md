# 06 — Formula Language (Lotion Formula)

Our own expression language, designed for spreadsheet-like familiarity. Clean-room: designed from general spreadsheet/expression-language conventions, not copied from any product's grammar or function list text.

## Types

`number` (IEEE double), `text`, `boolean`, `date` (instant + optional end, with `hasTime`), `list<T>`, `page` (row reference), `empty`. Every expression is type-checked when the formula is saved; the result type is fixed and shown in the property header. Runtime errors produce an `error` value displayed in the cell, never crash a view.

## Syntax

```
expr     := literal | prop | call | expr op expr | unary expr | "(" expr ")" | list | lambda
prop     := prop("Name") | identifier-in-brackets  e.g.  [Due date]
call     := name "(" args ")"  |  expr "." name "(" args ")"     -- method style: [Tags].length()
lambda   := (x) => expr  | current                              -- for map/filter/some/every/find
list     := "[" expr ("," expr)* "]"
op       := + - * / % ^  == != < <= > >=  and or  ??  &         -- & = text concat
let      := let(name, value, body)  /  lets(n1, v1, n2, v2, body)
```

Arabic property names and Arabic text literals are fully supported; the editor resolves direction per token so mixed expressions stay readable.

## Function set (0.1)

- **Logic:** `if`, `ifs`, `and`, `or`, `not`, `empty`, `coalesce`.
- **Math:** `abs`, `round(n, digits)`, `ceil`, `floor`, `sqrt`, `pow`, `exp`, `ln`, `log10`, `log2`, `min`, `max`, `sum`, `mean`, `median`, `mod`, `sign`, `pi`, `e`, `toNumber`, `format(n, pattern)`.
- **Text:** `length`, `concat`, `join`, `split`, `slice`, `contains`, `startsWith`, `endsWith`, `replace`, `replaceAll` (RE2-safe regex), `test`, `match`, `lower`, `upper`, `trim`, `padStart`, `repeat`, `link(label, url)`, `style(text, ...marks)`.
- **Date:** `now`, `today`, `date(y,m,d)`, `dateAdd(d, n, unit)`, `dateSubtract`, `dateBetween(a, b, unit)`, `dateRange(start, end)`, `dateStart`, `dateEnd`, `formatDate(d, pattern, locale)`, `parseDate`, `year`, `month`, `day`, `week`, `weekday`, `hour`, `minute`, `timestamp`, `fromTimestamp`. Hijri display via `formatDate(d, pattern, "ar-SA-u-ca-islamic")`.
- **Lists:** `at`, `first`, `last`, `length`, `sort`, `reverse`, `unique`, `includes`, `find`, `findIndex`, `filter`, `map`, `some`, `every`, `flat`, `concat`, `slice`, `count`.
- **Pages/relations:** `[Relation]` yields `list<page>`; `page.prop("Name")` reads a related row's property, enabling multi-hop expressions (`[Project].first().prop("Owner")`); `id()`, `createdTime()`, `lastEditedTime()`.

`now()` and `today()` make a formula **time-dependent**: such properties are recomputed at view render and on a minute/day tick, and are not used in persistent indexes.

## Evaluation and storage

- Parsed to an AST (hand-written Pratt parser), type-checked, then compiled to a closure tree. No `eval`, no JS access, deterministic except time functions.
- Results of non-time-dependent formulas and all rollups are **materialized** in `computed_value` (03-DATA-MODEL) so filters, sorts and groups run as indexed SQL.
- Dependencies (properties, relations, other formulas, rollups) are extracted from the AST into `prop_dependency`; the recompute engine (02-ARCHITECTURE §Computed values) uses them.
- Limits: AST ≤ 2,000 nodes, evaluation ≤ 10 ms per row (exceeding returns `error: too complex`), relation traversal depth ≤ 3 hops.

## Acceptance

AC-DB-005 and AC-DB-006 own this document. Tests: one golden file per function family (`tests/unit/formula/*.golden.json`: expression, row fixture, expected value/type/error), parser fuzzing (fast-check), and type-checker negative cases.
