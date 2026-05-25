===description===
No crash when comparing illegitimate callable
===file===
<?php
class C {}

function foo() : C {
    return fn (int $i) => "";
}
===expect===
InvalidReturnStatement
===ignore===
TODO
