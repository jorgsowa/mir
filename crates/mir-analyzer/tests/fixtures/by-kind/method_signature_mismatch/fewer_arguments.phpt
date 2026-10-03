===description===
Fewer arguments
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public function fooFoo(int $a, bool $b): void {

    }
}

class B extends A {
    public function fooFoo(int $a): void {
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method B::foofoo() signature mismatch: method has fewer parameters (1) than parent A::foofoo() (2)

    }
}
===expect===
