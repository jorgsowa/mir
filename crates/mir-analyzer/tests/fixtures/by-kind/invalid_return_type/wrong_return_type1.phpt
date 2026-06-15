===description===
Wrong return type1
===file===
<?php
function fooFoo(): string {
    return 5;
}
===expect===
InvalidReturnType@3:4-3:13: Return type '5' is not compatible with declared 'string'
