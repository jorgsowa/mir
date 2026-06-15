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
}
===expect===
PossiblyUndefinedVariable@6:11-6:16: Variable $last might not be defined
