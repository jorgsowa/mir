===description===
parenthesized variable-variable with known variable name

===file===
<?php
function test() {
    $a = 'b';
    $b = 'value';
    return ${$a};
}
===expect===
