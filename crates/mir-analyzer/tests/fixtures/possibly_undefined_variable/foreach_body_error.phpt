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
PossiblyUndefinedVariable@6:11: Variable $last might not be defined
===ignore===
TODO
