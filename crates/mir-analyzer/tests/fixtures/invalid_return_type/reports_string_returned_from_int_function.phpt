===source===
<?php
function f(): int {
    return 'hello';
}
===expect===
InvalidReturnType: Return type '"hello"' is not compatible with declared 'int'
