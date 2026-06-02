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
    echo $a;
  }
}
===expect===
MethodSignatureMismatch@8:2-8:41: Method B::foo() signature mismatch: parameter $a must not be passed by reference to match parent A::foo()
