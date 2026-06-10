===description===
Disallow subclasses for non inherited method params
===ignore===
TODO
===file===
<?php
class A {}
class B extends A {
  public function bar(): void {}
}
class C extends A {
  public function bar(): void {}
}

class D {
  public function foo(A $a): void {}
}

class E extends D {
  /** @param B|C $a */
  public function foo(A $a): void {
    $a->bar();
  }
}
===expect===
