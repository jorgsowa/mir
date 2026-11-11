===description===
A non-empty-string may be "0", which a non-falsy-string parameter rejects.
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

/**
 * @param non-empty-string $name
 * @param non-empty-string|false $found
 */
function run(string $name, $found, string $plain): void {
    takesNonFalsy($name);
//                ^^^^^ ArgumentTypeCoercion: Argument $value of takesNonFalsy() expects 'non-falsy-string', got 'non-empty-string' — coercion may fail at runtime
    takesNonFalsy($plain);
//                ^^^^^^ ArgumentTypeCoercion: Argument $value of takesNonFalsy() expects 'non-falsy-string', got 'string' — coercion may fail at runtime
    if ($found !== false) {
        takesNonFalsy($found);
//                    ^^^^^^ ArgumentTypeCoercion: Argument $value of takesNonFalsy() expects 'non-falsy-string', got 'non-empty-string' — coercion may fail at runtime
    }
}
