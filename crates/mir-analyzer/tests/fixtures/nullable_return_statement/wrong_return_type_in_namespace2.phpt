===description===
Wrong return type in namespace2
===file===
<?php
namespace bar;

function fooFoo(): string {
    return rand(0, 5) ? "hello" : null;
}
===expect===
NullableReturnStatement
===ignore===
TODO
