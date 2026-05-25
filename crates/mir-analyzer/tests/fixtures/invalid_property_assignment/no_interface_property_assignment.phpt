===description===
No interface property assignment
===file===
<?php
interface A { }

function fooFoo(A $a): void {
    $a->bar = 5;
}
===expect===
NoInterfaceProperties
===ignore===
TODO
