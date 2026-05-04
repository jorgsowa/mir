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
UndefinedVariable@5:11: Variable $value is not defined
