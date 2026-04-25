===file===
<?php
function f(): int {
    return;
}
===expect===
InvalidReturnType: Return type 'void' is not compatible with declared 'int'
