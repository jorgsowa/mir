===file===
<?php
class Foo {
    public function bar(int $_unused): int {
        return 42;
    }
}
===expect===
UnusedParam: Parameter $_unused is never used
