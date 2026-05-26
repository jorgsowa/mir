===description===
Incorrect callable param default
===file===
<?php
class A {
    public function foo(callable $_a = "strlen"): void {}
}

===expect===
InvalidParamDefault
===ignore===
TODO
