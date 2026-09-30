===description===
Wrong return type1
===file===
<?php
function fooFoo(): string {
    return 5;
//  ^^^^^^^^^ InvalidReturnType: Return type '5' is not compatible with declared 'string'
}
===expect===
