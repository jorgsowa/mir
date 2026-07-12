===description===
a private static method used only as a "Class::method" string callback to array_filter must not be reported unused
===config===
suppress=
===file===
<?php
class Filters {
    private static function keep(int $x): bool { return $x > 0; }

    public static function run(array $items): array {
        return array_filter($items, 'Filters::keep');
    }
}
===expect===
