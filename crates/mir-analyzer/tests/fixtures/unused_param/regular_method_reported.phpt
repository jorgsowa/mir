===source===
<?php
class Foo {
    public function bar(int $x): int {
        return 42;
    }
}
===expect===
UnusedParam: Parameter $x is never used
