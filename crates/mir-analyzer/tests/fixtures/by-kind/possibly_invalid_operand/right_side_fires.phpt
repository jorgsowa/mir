===description===
PossiblyInvalidOperand fires when the right operand's union contains an array.
===file===
<?php
function scale(int $n, int|array $data): void {
    $_ = $n * $data;
//       ^^^^^^^^^^ PossiblyInvalidOperand: Operator '*' might not be supported between 'int' and 'int|array'
}
