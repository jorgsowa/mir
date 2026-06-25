===description===
InvalidStaticInvocation does NOT fire for methods whose names start with __ (magic methods are exempt).
===file===
<?php
class Serializer {
    public function __toString(): string { return ""; }
}

Serializer::__toString();
===expect===
