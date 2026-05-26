===description===
isset short-circuit with && — numeric comparison on RHS
isset($max) && $value > $max: no UndefinedVariable on $max in the comparison operand
===file===
<?php
/** @param int|null $max */
function test(?int $max, int $value): bool {
    return isset($max) && $value > $max;
}
===expect===
