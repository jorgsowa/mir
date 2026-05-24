===description===
traitMethodMadePrivate
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
InaccessibleMethod
===ignore===
TODO
