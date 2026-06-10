===description===
Incorrect callable param default
===ignore===
TODO
===file===
<?php
class A {
    public function foo(callable $_a = "strlen"): void {}
}

===expect===
