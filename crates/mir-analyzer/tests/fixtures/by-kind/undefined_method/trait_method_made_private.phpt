===description===
A trait method aliased as private is accessible within the declaring class but not from subclasses.
===file===
<?php
trait T {
    public function foo() : void {
        echo "here";
    }
}

class C {
    use T {
        foo as private traitFoo;
    }

    public function bar() : void {
        $this->traitFoo();
    }
}

class D extends C {
    public function bar() : void {
        $this->traitFoo();
    }
}
===expect===
UndefinedMethod@20:8-20:25: Method C::traitFoo() does not exist
