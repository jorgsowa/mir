===description===
variable-variable operand in assignment target

===config===
suppress=MissingReturnType
===file===
<?php
function test() {
    $key = 'value';
    $$key = 42;
    return $value;
}
===expect===
