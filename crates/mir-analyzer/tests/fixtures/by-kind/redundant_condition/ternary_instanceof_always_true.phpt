===description===
`instanceof` an ancestor or implemented interface in a ternary condition is always true.
===file===
<?php
interface Shape {}
class Polygon implements Shape {}
class Square extends Polygon {}

function parentClass(Square $square): string {
    return $square instanceof Polygon ? 'polygon' : 'other';
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always true, so the else branch is never reached
}

function implementedInterface(Square $square): string {
    return $square instanceof Shape ? 'shape' : 'other';
//         ^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always true, so the else branch is never reached
}
