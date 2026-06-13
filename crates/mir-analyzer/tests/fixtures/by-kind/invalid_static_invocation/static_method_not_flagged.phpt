===description===
InvalidStaticInvocation does NOT fire when calling a static method statically.
===config===
suppress=UnusedVariable
===file===
<?php
class Math {
    public static function double(int $n): int {
        return $n * 2;
    }
}

$result = Math::double(5);
===expect===
