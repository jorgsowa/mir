===description===
Closure param seeded from the array element type is a scalar receiver.
===file===
<?php
function f(): array {
    return array_map(fn($i) => $i->go(), [1, 2]);
//                             ^^^^^^^^ InvalidMethodCall: Cannot call method go() on non-object type '1|2'
}
