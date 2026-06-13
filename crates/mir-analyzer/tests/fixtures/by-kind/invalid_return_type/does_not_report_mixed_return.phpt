===description===
does not report mixed return
===config===
suppress=MixedAssignment,MixedReturnStatement
===file===
<?php
function f(): int {
    $x = json_decode('{}');
    return $x;
}
===expect===
