===description===
One param
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
