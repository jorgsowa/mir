===description===
No interface property assignment
===ignore===
TODO
===file===
<?php
interface A { }

function fooFoo(A $a): void {
    $a->bar = 5;
}
===expect===
