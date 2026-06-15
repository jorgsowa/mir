===description===
Wrong return type2
===file===
<?php
function fooFoo(): string {
    return rand(0, 5) ? "hello" : null;
}
===expect===
NullableReturnStatement@3:4-3:39: Return type '"hello"|null' is not compatible with declared 'string'
