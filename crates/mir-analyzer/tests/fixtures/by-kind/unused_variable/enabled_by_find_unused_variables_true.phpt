===description===
UnusedVariable and UnusedForeachValue are reported when findUnusedVariables is true
===config===
<mir>
  <findUnusedVariables>true</findUnusedVariables>
</mir>
===file===
<?php
/** @param array<string, int> $items */
function total(array $items): int {
    $unused = count($items);
//  ^^^^^^^ UnusedVariable: Variable $unused is never read
    foreach ($items as $k => $v) {
//                           ^^ UnusedForeachValue: Foreach value $v is never read
        echo $k;
    }
    return 1;
}
===expect===
