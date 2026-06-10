===description===
Inherit param types
===ignore===
TODO
===file===
<?php
class A {
  /**
   * @param string $bar
   * @return void
   */
  public function foo($bar) {
    echo $bar;
  }
}

class B extends A {
  public function foo($bar) {
    echo "hello " . $bar;
  }
}

(new B)->foo(new stdClass);
===expect===
