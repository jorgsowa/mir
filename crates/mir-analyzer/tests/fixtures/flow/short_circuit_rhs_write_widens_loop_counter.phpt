===description===
A write in the RHS of `&&` joins with the RHS-skipped state, so a loop counter incremented there widens across iterations instead of staying a literal.
===file===
<?php
/** @param list<bool> $items */
function capped(array $items): void {
    $accepted = 0;
    foreach ($items as $item) {
        if ($item && ++$accepted === 50) {
            break;
        }
    }
}
