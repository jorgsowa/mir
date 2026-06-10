===description===
No interface property fetch
===file===
<?php
interface A { }

function fooFoo(A $a): void {
    if ($a->bar) {

    }
}
===expect===
