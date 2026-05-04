===description===
variable-variable operand in assignment target

===file===
<?php
function test() {
    $key = 'value';
    $$key = 42;
    return $value;
}
===expect===
