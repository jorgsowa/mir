===description===
reports bare return from non void
===file===
<?php
function f(): int {
    return;
}
===expect===
InvalidReturnType@3:4-3:11: Return type 'void' is not compatible with declared 'int'
