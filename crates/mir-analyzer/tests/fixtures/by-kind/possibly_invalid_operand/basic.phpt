===description===
PossiblyInvalidOperand fires when an operand in a union contains an array or object.
===file===
<?php
function double(int|array $a): int {
    return $a * 2;
}
===expect===
PossiblyInvalidOperand@3:12-3:18: Operator '*' might not be supported between 'int|array<mixed, mixed>' and '2'
