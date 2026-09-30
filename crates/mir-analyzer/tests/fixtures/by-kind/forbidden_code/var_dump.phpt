===description===
ForbiddenCode fires when calling var_dump.
===file===
<?php
function debug(mixed $v): void {
    var_dump($v);
//  ^^^^^^^^^^^^ ForbiddenCode: Use of var_dump is forbidden
}
===expect===
