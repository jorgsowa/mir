===description===
UnusedClass does NOT fire for abstract classes — only final non-abstract classes
are checked, since abstract classes can be referenced via type hints or inheritance
in ways the reference tracker may not capture.
===file===
<?php
/** @psalm-internal */
abstract class NeverExtended {
    abstract public function work(): void;
}

===expect===
