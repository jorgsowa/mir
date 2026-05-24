===description===
reports bare return from non void
===file===
<?php
function f(): int {
    return;
}
===expect===
InvalidReturnType@3:5: Return type 'void' is not compatible with declared 'int'
