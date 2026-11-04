===description===
A defensive `false === hrtime()` guard isn't an impossible comparison
===file===
<?php
function f(): int {
    if (false === $t = hrtime()) {
        return 0;
    }
    return $t[0];
}
