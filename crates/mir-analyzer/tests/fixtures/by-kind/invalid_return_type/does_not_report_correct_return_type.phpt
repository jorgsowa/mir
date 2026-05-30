===description===
does not report correct return type
===file===
<?php
function f(): int {
    return 42;
}
===expect===
