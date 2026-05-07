===description===
underscore param reported
===file===
<?php
class Foo {
    public function bar(int $_unused): int {
        return 42;
    }
}
===expect===
UnusedParam@3:24: Parameter $_unused is never used
