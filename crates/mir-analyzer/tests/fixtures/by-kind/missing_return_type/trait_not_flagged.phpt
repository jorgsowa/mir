===description===
MissingReturnType does NOT fire for trait methods — the check applies only to
interface methods and top-level functions.
===file===
<?php
trait MyTrait {
    public function noReturn() { return 1; }
    abstract public function abstractNoReturn();
}
===expect===
