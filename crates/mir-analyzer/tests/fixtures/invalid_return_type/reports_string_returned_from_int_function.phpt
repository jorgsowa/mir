===description===
Returning a string literal from a function declared to return int reports InvalidReturnType.
===file===
<?php
function f(): int {
    return 'hello';
}
===expect===
InvalidReturnType: Return type '"hello"' is not compatible with declared 'int'
