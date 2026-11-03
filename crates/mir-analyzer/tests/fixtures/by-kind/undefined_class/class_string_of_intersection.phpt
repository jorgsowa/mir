===description===
A class-string of an intersection or union names each member class, not one class called `A&B`.
===file===
<?php
interface TimeBased {}
abstract class Uuid {
    public static function generate(): string { return ''; }
}
final class Factory {
    /** @param class-string<Uuid&TimeBased> $class */
    public function __construct(private string $class) {}

    public function create(): string {
        $class = $this->class;
        return $class::generate();
    }
}

abstract class Ulid {
    public static function generate(): string { return ''; }
}

/** @param class-string<Uuid|Ulid> $class */
function generateAny(string $class): mixed {
    return $class::generate();
}
