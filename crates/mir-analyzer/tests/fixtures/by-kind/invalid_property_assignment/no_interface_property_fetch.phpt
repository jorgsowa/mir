===description===
Property fetch on a bare interface with no declared members flags
NoInterfaceProperties, not InvalidPropertyAssignment/silence.
===file===
<?php
interface A { }

function fooFoo(A $a): void {
    if ($a->bar) {
//          ^^^ NoInterfaceProperties: Property $bar is not defined on this interface

    }
}
