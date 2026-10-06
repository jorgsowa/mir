===description===
underscore param reported
===file===
<?php
class Foo {
    public function bar(int $_unused): int {
//                      ^^^^^^^^^^^^ UnusedParam: Parameter $_unused is never used
        return 42;
    }
}
