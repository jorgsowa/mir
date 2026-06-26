===description===
MissingReturnType does NOT fire for abstract class methods — the check applies only
to interface methods and top-level functions.
===file===
<?php
abstract class Base {
    abstract public function abstractNoReturn();
    abstract protected function abstractProtectedNoReturn();
}
===expect===
