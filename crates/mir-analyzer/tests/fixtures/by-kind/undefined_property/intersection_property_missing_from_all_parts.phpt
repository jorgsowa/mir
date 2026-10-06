===description===
A property missing from every part of an intersection type is UndefinedProperty.
===file===
<?php
interface CountableIface {
    public function count(): int;
}
class Foo {
    public string $known = 'x';
}
function useIt(object $x): void {
    if ($x instanceof Foo && $x instanceof CountableIface) {
        echo $x->nonexistentProp;
//               ^^^^^^^^^^^^^^^ UndefinedProperty: Property Foo&CountableIface::$nonexistentProp does not exist
    }
}
