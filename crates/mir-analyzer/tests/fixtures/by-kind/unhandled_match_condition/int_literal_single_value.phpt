===description===
UnhandledMatchCondition fires for a single int literal subject with no matching arm.
===config===
<mir>
  <issueHandlers>
    <TypeDoesNotContainType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param 42 $n */
function check(int $n): string {
    return match($n) {
//         ^ +2:5 UnhandledMatchCondition: Unhandled match condition: 42
        0 => "zero",
    };
}
