===description===
Property assignment on a bare interface with no declared members flags
NoInterfaceProperties.
===file===
<?php
interface A { }

function fooFoo(A $a): void {
    $a->bar = 5;
//  ^^^^^^^^^^^ NoInterfaceProperties: Property $bar is not defined on this interface
}
