===source===
<?php
function f(): int {
    return 'hello';
}
===expect===
InvalidReturnType: return 'hello';
