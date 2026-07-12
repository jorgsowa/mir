===description===
UnusedClass does NOT fire for a class that is instantiated or referenced.
===config===
suppress=
===file===
<?php
/** @psalm-internal */
final class Used {
    public function hello(): string { return "hi"; }
}

new Used();

===expect===
