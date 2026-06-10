===description===
Class using trait
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
