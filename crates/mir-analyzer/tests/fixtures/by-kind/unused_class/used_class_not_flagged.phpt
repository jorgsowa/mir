===description===
UnusedClass does NOT fire for a class that is instantiated or referenced.
===config===
suppress=UnusedVariable
===file===
<?php
/** @psalm-internal */
final class Used {
    public function hello(): string { return "hi"; }
}

$obj = new Used();

===expect===
