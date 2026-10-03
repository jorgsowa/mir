===description===
Mismatching interface method signature in implementer
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface A {
    public function fooFoo(int $a, int $b): void;
}

trait T {
    public function fooFoo(int $a, int $b): void {
    }
}

class B implements A {
    use T;

    public function fooFoo(int $a): void {
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method B::foofoo() signature mismatch: method has fewer parameters (1) than parent A::foofoo() (2)
    }
}
===expect===
