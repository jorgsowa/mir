===description===
Undefined method on parent call with method exists on self
===config===
suppress=MixedReturnStatement
===file===
<?php
class A {}
class B extends A {
    public function foo(): string {
        return parent::foo();
    }
}
===expect===
UndefinedMethod@5:16-5:29: Method A::foo() does not exist
