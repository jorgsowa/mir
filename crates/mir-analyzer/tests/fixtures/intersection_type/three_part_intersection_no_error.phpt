===description===
three part intersection no error
===file===
<?php
interface A {}
interface B {}
interface C {}

function f(A&B&C $x): void {
    $_ = $x;
}
===expect===
===ignore===
TODO
