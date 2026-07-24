===description===
Contrast case: an override that DOES re-declare @pure on itself must
stay unflagged.
===file===
<?php
interface Calculator {
    /** @pure */
    public function add(int $a, int $b): int;
}
class StillPure implements Calculator {
    /** @pure */
    public function add(int $a, int $b): int {
        return $a + $b;
    }
}
===expect===
