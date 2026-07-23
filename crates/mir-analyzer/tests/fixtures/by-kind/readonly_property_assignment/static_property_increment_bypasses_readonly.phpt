===description===
`Frozen::$hits++` mutates a readonly static property just as much as a
plain assignment would, but unary.rs's `++`/`--` handling had no
`StaticPropertyAccess` arm at all -- readonly enforcement was completely
invisible for this operand shape.
===file===
<?php
class Frozen {
    /** @readonly */
    public static int $hits = 0;
}
function tick(): void {
    Frozen::$hits++;
}
===expect===
ReadonlyPropertyAssignment@7:4-7:17: Cannot assign to readonly property Frozen::$hits outside of constructor
