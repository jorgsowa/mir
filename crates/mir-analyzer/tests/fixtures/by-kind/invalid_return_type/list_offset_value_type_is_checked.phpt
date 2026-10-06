===description===
list offset value type is checked
===file===
<?php
function first(): int {
    $values = ['alpha', 'beta'];
    return $values[0];
//  ^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type '"alpha"' is not compatible with declared 'int'
}
