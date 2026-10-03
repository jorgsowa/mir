===description===
Negative control for the `static $x = null;` + `@var` fix: with NO preceding `@var`
docblock, a static var's type must still come from its literal initializer as before
— `static $x = null;` alone still makes a later `$x !== null` check genuinely always
false, and that diagnostic must keep firing.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    static $x = null;
    if ($x !== null) {
//      ^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
        echo 'unreachable';
    }
}
===expect===
