===description===
clasginByRef
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
MethodSignatureMismatch
===ignore===
TODO
