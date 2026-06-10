===description===
No interface property fetch
===ignore===
TODO
===file===
<?php
interface A { }

function fooFoo(A $a): void {
    if ($a->bar) {

    }
}
===expect===
