===description===
A plain `break;` (level 1) only exits the innermost `foreach`; the outer
`while (true)` still never exits normally, so code after it is still
correctly unreachable.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(array $matrix): void {
    while (true) {
        foreach ($matrix as $cell) {
            if ($cell === 0) {
                break;
            }
        }
    }
    echo "after";
//  ^^^^^^^^^^^^^ UnreachableCode: Unreachable code detected
}
===expect===
