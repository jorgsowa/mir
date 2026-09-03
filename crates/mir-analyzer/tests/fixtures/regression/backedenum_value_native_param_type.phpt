===description===
P20: A native `BackedEnum`-typed param must resolve `->value` — BackedEnum extends UnitEnum and
redeclares `$value` as `int|string`, so both interfaces' member-collection loops need the
property arm, not just one.
===file===
<?php

function backingValue(BackedEnum $e): int|string
{
    return $e->value;
}
===expect===
