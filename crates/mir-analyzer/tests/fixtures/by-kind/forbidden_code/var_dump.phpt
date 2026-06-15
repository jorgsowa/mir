===description===
ForbiddenCode fires when calling var_dump.
===file===
<?php
function debug(mixed $v): void {
    var_dump($v);
}
===expect===
ForbiddenCode@3:4-3:16: Use of var_dump is forbidden
