===description===
PossiblyInvalidOperand also fires for object in arithmetic.
===file===
<?php
class Box {}

function scale(int|Box $x): int {
    return $x * 3;
}
===expect===
PossiblyInvalidOperand@5:12-5:18: Operator '*' might not be supported between 'int|Box' and '3'
