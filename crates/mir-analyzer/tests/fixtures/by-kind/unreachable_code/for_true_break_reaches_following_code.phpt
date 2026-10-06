===description===
A break in a for(;true;) loop makes the code after it reachable.
===file===
<?php
function a(int $limit): int {
    for ($i = 0; true; ++$i) {
        if ($i > $limit) {
            break;
        }
    }
    return $i;
}
function b(): int {
    for ($i = 0; false; ++$i) {
    }
    return $i;
}
