===description===
P20: A native `UnitEnum`-typed param must resolve `->name` directly, with no `instanceof`
narrowing involved — covers the case where the interface itself is the declared param type.
===file===
<?php

function label(UnitEnum $e): string
{
    return $e->name;
}
===expect===
