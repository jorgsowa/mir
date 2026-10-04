===description===
UnusedVariable and UnusedForeachValue are hidden when findUnusedVariables is false
===config===
<mir>
  <findUnusedVariables>false</findUnusedVariables>
</mir>
===file===
<?php
/** @param array<string, int> $items */
function total(array $items): int {
    $unused = count($items);
    foreach ($items as $k => $v) {
        echo $k;
    }
    return 1;
}
===expect===
