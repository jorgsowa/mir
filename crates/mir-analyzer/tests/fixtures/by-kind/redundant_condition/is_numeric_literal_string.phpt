===description===
is_numeric on a literal-string "123" is always true (numeric literal string eliminated from false branch);
is_numeric on "hello" is always false (non-numeric literal string eliminated from true branch)
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test_always_true(string $s): void {
    $s = "123";
    if (is_numeric($s)) {
//      ^^^^^^^^^^^^^^ RedundantCondition: Condition is always true, so the check is redundant
        // always taken
    }
}
function test_always_false(string $s): void {
    $s = "hello";
    if (is_numeric($s)) {
//      ^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
        // never taken
    }
}
