===description===
does not report null check on nullable
===file===
<?php
function f(?string $x): void {
    if ($x === null) {}
}
===expect===
