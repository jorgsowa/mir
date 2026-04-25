===file===
<?php
function f(): string {
    return null;
}
===expect===
InvalidReturnType: Return type 'null' is not compatible with declared 'string'
