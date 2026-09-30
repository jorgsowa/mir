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
//             ^^^^^^^^^^^^^ UndefinedMethod: Method A::foo() does not exist
    }
}
===expect===
