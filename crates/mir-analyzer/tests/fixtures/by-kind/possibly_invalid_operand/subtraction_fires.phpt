===description===
PossiblyInvalidOperand fires for subtraction when a union member is an array.
===file===
<?php
function diff(int|array $a, int $b): int {
    return $a - $b;
}
===expect===
PossiblyInvalidOperand@3:11-3:18: Operator '-' might not be supported between 'int|array' and 'int'
