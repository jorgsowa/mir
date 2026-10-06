===description===
findUnusedCode implies unused-variable reporting even when findUnusedVariables is false
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
  <findUnusedVariables>false</findUnusedVariables>
</mir>
===file===
<?php
function total(array $items): int {
    $unused = count($items);
//  ^^^^^^^ UnusedVariable: Variable $unused is never read
    return 1;
}
total([]);
