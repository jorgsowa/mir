===description===
foreach body error
===file===
<?php
function foo(array $items): string {
    foreach ($items as $item) {
        $last = $item;
    }
    return $last;
}
===expect===
PossiblyUndefinedVariable@6:12-6:17: Variable $last might not be defined
