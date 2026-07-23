===description===
@psalm-assert-if-true on a method call never narrowed when the receiver
was an intersection type (`Foo&Bar`) -- method_call_receiver_fqcn's match
only handled a single concrete class/self/static/parent atom, falling
through to None for a TIntersection even though ordinary method-call
resolution (call/method.rs) already dispatches through it fine.
===config===
suppress=MissingConstructor
===file===
<?php
interface Bar {}
class Validator implements Bar {
    /**
     * @param mixed $p
     * @psalm-assert-if-true int $p
     */
    public function isInt($p): bool {
        return is_int($p);
    }
}
/**
 * @param mixed $p
 */
function doWork(Validator&Bar $obj, $p): void {
    if ($obj->isInt($p)) {
        strlen($p);
    }
}
===expect===
ArgumentTypeCoercion@17:15-17:17: Argument $string of strlen() expects 'string', got 'int' — coercion may fail at runtime
