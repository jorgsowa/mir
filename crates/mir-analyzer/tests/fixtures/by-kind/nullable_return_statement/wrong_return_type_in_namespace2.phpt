===description===
Wrong return type in namespace2
===file===
<?php
namespace bar;

function fooFoo(): string {
    return rand(0, 5) ? "hello" : null;
}
===expect===
NullableReturnStatement@5:5-5:40: Return type '"hello"|null' is not compatible with declared 'string'
