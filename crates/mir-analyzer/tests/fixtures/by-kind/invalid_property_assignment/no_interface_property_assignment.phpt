===description===
Property assignment on a bare interface with no declared members flags
NoInterfaceProperties.
===file===
<?php
interface A { }

function fooFoo(A $a): void {
    $a->bar = 5;
}
===expect===
NoInterfaceProperties@5:4-5:15: Property $bar is not defined on this interface
