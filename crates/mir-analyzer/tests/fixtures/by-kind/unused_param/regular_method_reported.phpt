===description===
regular method reported
===file===
<?php
class Foo {
    public function bar(int $x): int {
        return 42;
    }
}
===expect===
UnusedParam@3:25-3:31: Parameter $x is never used
