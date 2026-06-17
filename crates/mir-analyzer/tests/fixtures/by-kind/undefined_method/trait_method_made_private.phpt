===description===
Trait method aliased as private. The alias is resolved so no UndefinedMethod is raised.
Private-from-subclass visibility check (line 20) is not yet implemented.
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
        $this->traitFoo(); // should eventually fail: private method of C
    }
}
===expect===
