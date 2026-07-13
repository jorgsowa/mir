===description===
Sibling of intersection_method_missing_from_all_parts: a method found on one part is not flagged.
===file===
<?php
interface CountableIface {
    public function count(): int;
}
class Foo {
    public function bar(): void {}
}
function useIt(object $x): void {
    if ($x instanceof Foo && $x instanceof CountableIface) {
        $x->bar();
        $x->count();
    }
}
===expect===
