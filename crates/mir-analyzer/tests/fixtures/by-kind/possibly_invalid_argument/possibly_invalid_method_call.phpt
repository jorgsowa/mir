===description===
Possibly invalid method call
===file===
<?php
class A1 {
    public function methodOfA(): void {
    }
}

/** @param A1|string $x */
function example($x, bool $isObject) : void {
    if ($isObject) {
        $x->methodOfA();
    }
}
===expect===
PossiblyInvalidMethodCall
===ignore===
TODO
