===source===
<?php
class Foo {
    public function bar(int $_unused): int {
        return 42;
    }
}
===expect===
UnusedParam: $_unused
