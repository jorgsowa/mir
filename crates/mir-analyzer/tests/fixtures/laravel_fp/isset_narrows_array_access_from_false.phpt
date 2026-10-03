===description===
Regression (laravel/framework): `preg_split()` returns array|false; an access
guarded by `isset($matches[1])` is safe. isset() on an array-access target now
narrows false/null out of the base variable, so mir no longer emits
PossiblyInvalidArrayAccess (Console\Parser).
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function firstWord(string $token): string {
    $matches = preg_split('/\s+/', $token);
    if (isset($matches[1])) {
        return $matches[1];
    }
    return '';
}
===expect===
