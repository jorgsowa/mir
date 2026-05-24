===description===
wrongReturnType2
===file===
<?php
function fooFoo(): string {
    return rand(0, 5) ? "hello" : null;
}
===expect===
NullableReturnStatement
===ignore===
TODO
