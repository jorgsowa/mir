===description===
PossiblyInvalidOperand fires when an operand in a union contains an array or object.
===file===
<?php
function double(int|array $a): int {
    return $a * 2;
//         ^^^^^^ PossiblyInvalidOperand: Operator '*' might not be supported between 'int|array' and '2'
}
