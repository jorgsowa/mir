===description===
A trait method made protected via `as protected` (no rename) is accessible within the declaring class and subclasses but not from outside.
===file===
<?php
trait T {
    public function foo() : void {
        echo "here";
    }
}

class C {
    use T {
        foo as protected;
    }
}

class D extends C {
    public function bar() : void {
        $this->foo();
    }
}

function callFromOutside(C $c): void {
    $c->foo();
}
===expect===
UndefinedMethod@21:4-21:13: Method C::foo() does not exist
