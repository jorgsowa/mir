===description===
Reconcile after interface instanceof
===ignore===
TODO
===file===
<?php
interface Base {}

interface E extends Base {
    public function bar() : void;
}

function foobar(Base $foo) : void {
    if ($foo instanceof E) {
        $foo->bar();
    }

    $foo->bar();
}
===expect===
