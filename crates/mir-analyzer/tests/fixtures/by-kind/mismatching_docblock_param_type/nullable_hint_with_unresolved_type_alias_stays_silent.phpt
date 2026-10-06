===description===
Regression guard for the nullable-hint-vs-docblock check: a `@psalm-type`
alias name (e.g. `MaybeUser`) referenced as the docblock `@param` type isn't
expanded before this check runs, so it may already include `null`
internally even though the raw atom has no literal `TNull` — the check
must stay silent rather than treating every alias as a nullability
contradiction.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @psalm-type MaybeString = string|null
 * @param MaybeString $a
 */
function viaAlias(?string $a): void {}
