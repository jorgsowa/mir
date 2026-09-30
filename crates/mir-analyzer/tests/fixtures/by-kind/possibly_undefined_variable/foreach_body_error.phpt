===description===
foreach body error
===config===
suppress=MixedAssignment,MixedReturnStatement
===file===
<?php
function foo(array $items): string {
    foreach ($items as $item) {
        $last = $item;
    }
    return $last;
//         ^^^^^ PossiblyUndefinedVariable: Variable $last might not be defined
}
===expect===
