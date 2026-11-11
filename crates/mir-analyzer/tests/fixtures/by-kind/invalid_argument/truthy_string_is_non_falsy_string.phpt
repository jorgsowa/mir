===description===
A string that passed a truthiness check is neither "" nor "0", so it is a non-falsy-string.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param non-falsy-string $value */
function takesNonFalsy(string $value): void {}

/** @param non-empty-string $name */
function run(string $plain, string $name, ?string $optional): void {
    if ($plain) {
        takesNonFalsy($plain);
    }
    if ($name) {
        takesNonFalsy($name);
    }
    if (!empty($optional)) {
        takesNonFalsy($optional);
    }
}
