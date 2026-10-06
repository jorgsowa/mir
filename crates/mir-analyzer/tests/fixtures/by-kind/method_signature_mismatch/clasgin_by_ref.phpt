===description===
Clasgin by ref
===file===
<?php
class A {
  public function foo(string $a): void {
    echo $a;
  }
}
class B extends A {
  public function foo(string &$a): void {
//^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method B::foo() signature mismatch: parameter $a must not be passed by reference to match parent A::foo()
    echo $a;
  }
}
