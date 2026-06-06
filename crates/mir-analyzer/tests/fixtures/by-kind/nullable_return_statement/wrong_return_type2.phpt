===description===
Wrong return type2
===file===
<?php
function fooFoo(): string {
    return rand(0, 5) ? "hello" : null;
}
===expect===
NullableReturnStatement@3:5-3:40: Return type '"hello"|null' is not compatible with declared 'string'
