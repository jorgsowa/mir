===description===
Hiding unused-variable kinds does not hide unrelated diagnostics in the same function
===config===
<mir>
  <findUnusedVariables>false</findUnusedVariables>
</mir>
===file===
<?php
function total(bool $flag): int {
    $unused = 1;
    if ($flag) {
        $maybe = 2;
    }
    return $maybe;
//         ^^^^^^ PossiblyUndefinedVariable: Variable $maybe might not be defined
}
