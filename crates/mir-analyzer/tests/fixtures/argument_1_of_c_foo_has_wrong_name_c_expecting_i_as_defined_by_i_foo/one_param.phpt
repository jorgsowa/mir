===description===
oneParam
===file===
<?php
interface I {
    /**
     * @param array $i
     */
    public function foo(array $i) : void;
}

class C implements I {
    public function foo(array $c) : void {
        return;
    }
}
===expect===
Argument 1 of C::foo has wrong name $c, expecting $i as defined by I::foo
===ignore===
TODO
