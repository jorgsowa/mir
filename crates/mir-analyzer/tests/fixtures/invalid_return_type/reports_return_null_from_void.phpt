===source===
<?php
function f(): void {
    return null;
}
===expect===
InvalidReturnType: Return type 'null' is not compatible with declared 'void'
