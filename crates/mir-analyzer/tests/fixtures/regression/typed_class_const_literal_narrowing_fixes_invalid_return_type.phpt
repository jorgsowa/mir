===description===
Same shared type-priority bug also surfaced as InvalidReturnType (Sector B7): a typed
class constant's discarded literal precision made an otherwise-exact return type look
too wide.
===config===
suppress=UnusedParam
===file===
<?php
final class Foo {
    private const int ID = 5;

    /** @return positive-int */
    public function id(): int {
        return self::ID;
    }
}

===expect===
