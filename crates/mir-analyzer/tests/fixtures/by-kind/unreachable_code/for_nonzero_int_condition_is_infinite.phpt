===description===
A for loop with a nonzero int condition is infinite; a parenthesized true too.
===file===
<?php
function a(int $limit): int {
    for ($i = 0; 1; ++$i) {
        if ($i > $limit) {
            return $i;
        }
    }
}
function b(int $limit): int {
    for ($i = 0; (true); ++$i) {
        if ($i > $limit) {
            return $i;
        }
    }
}
===expect===
