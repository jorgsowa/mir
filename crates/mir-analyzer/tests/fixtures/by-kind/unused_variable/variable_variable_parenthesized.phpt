===description===
parenthesized variable-variable with known variable name

===config===
suppress=MissingReturnType,UnusedVariable
===file===
<?php
function test() {
    $a = 'b';
    $b = 'value';
    return ${$a};
}
===expect===
