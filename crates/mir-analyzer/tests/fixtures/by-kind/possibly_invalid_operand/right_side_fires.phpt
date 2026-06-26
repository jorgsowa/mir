===description===
PossiblyInvalidOperand fires when the right operand's union contains an array.
===file===
<?php
function scale(int $n, int|array $data): void {
    $_ = $n * $data;
}
===expect===
PossiblyInvalidOperand@3:9-3:19: Operator '*' might not be supported between 'int' and 'int|array<mixed, mixed>'
