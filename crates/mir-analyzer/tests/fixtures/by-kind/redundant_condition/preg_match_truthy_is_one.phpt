===description===
After `if (preg_match(...))`, the result is known to be 1 (truthy int<0,1> = int<1,1>).
Checking `if ($r === 1)` in the true branch is therefore redundant.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(string $s): void {
    $r = preg_match('/foo/', $s);
    if ($r) {
        /** @mir-check $r is int<1, 1> */
        if ($r === 1) {
//          ^^^^^^^^ RedundantCondition: Condition is always true, so the check is redundant
            $_ = 'always here';
        }
    }
}
===expect===
