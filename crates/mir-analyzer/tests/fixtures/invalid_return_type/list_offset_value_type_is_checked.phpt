===file===
<?php
function first(): int {
    $values = ['alpha', 'beta'];
    return $values[0];
}
===expect===
InvalidReturnType: Return type '"alpha"' is not compatible with declared 'int'
