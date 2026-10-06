===description===
regular method reported
===file===
<?php
class Foo {
    public function bar(int $x): int {
//                      ^^^^^^ UnusedParam: Parameter $x is never used
        return 42;
    }
}
