===description===
a private static method used only as a "Class::method" string argument to Closure::fromCallable must not be reported unused
===config===
suppress=
===file===
<?php
class Filters {
    private static function keep(int $x): bool { return $x > 0; }

    public static function run(): \Closure {
        return \Closure::fromCallable('Filters::keep');
    }
}
===expect===
