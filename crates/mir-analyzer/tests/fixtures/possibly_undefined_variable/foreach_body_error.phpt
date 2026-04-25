===file===
<?php
function foo(array $items): string {
    foreach ($items as $item) {
        $last = $item;
    }
    return $last;
}
===expect===
PossiblyUndefinedVariable: Variable $last might not be defined
