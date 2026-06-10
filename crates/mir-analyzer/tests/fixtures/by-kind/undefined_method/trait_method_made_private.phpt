===description===
Trait method made private
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
        $this->traitFoo(); // should fail
    }
}
===expect===
UndefinedMethod@14:9-14:26: Method C::traitFoo() does not exist
UndefinedMethod@20:9-20:26: Method D::traitFoo() does not exist
