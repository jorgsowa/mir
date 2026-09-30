===description===
Wrong return type2
===file===
<?php
function fooFoo(): string {
    return rand(0, 5) ? "hello" : null;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type '"hello"|null' is not compatible with declared 'string'
}
===expect===
