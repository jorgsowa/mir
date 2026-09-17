===description===
A continue in an infinite loop re-enters that loop; it is not an exit path.
===file===
<?php
function neverReturns(): int {
    while (true) {
        continue;
    }
}
===expect===
