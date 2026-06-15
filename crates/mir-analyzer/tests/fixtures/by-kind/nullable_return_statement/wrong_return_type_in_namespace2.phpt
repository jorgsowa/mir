===description===
Wrong return type in namespace2
===file===
<?php
namespace bar;

function fooFoo(): string {
    return rand(0, 5) ? "hello" : null;
}
===expect===
NullableReturnStatement@5:4-5:39: Return type '"hello"|null' is not compatible with declared 'string'
