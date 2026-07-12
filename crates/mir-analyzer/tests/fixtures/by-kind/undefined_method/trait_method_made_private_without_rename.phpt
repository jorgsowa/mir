===description===
A trait method made private via `as private` (no rename) is accessible within the declaring class but not from subclasses.
===file===
<?php
trait T {
    public function foo() : void {
        echo "here";
    }
}

class C {
    use T {
        foo as private;
    }

    public function bar() : void {
        $this->foo();
    }
}

class D extends C {
    public function bar() : void {
        $this->foo();
    }
}
===expect===
UndefinedMethod@20:8-20:20: Method C::foo() does not exist
