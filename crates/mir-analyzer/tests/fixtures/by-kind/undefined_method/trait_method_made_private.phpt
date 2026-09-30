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
//      ^^^^^^^^^^^^^^^^^ UndefinedMethod: Method C::traitFoo() does not exist
    }
}
===expect===
