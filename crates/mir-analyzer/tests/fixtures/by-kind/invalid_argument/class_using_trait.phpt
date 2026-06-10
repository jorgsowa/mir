===description===
Class using trait
===ignore===
TODO
===file===
<?php
trait T {
    abstract public function f(): void;
}

class C {
    use T;

    public function f(): void {}
}

===expect===
