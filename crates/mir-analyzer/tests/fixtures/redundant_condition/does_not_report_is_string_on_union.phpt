===description===
does not report is string on union
===file===
<?php
function f(string|int $x): void {
    if (is_string($x)) {}
}
===expect===
===ignore===
TODO
