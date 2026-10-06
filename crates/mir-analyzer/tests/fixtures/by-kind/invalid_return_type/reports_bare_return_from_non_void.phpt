===description===
reports bare return from non void
===file===
<?php
function f(): int {
    return;
//  ^^^^^^^ InvalidReturnType: Return type 'void' is not compatible with declared 'int'
}
