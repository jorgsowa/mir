===description===
Incorrect callable param default
===config===
suppress=UnusedParam
===file===
<?php
class A {
    public function foo(callable $_a = "strlen"): void {}
}

===expect===
