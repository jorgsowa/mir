===description===
A docblock `Class::*` type is a union of constant values, so interpolating, concatenating or echoing it is not an implicit object cast.
===file===
<?php
namespace Lib;

final class Kind {
    public const A = 'a';
    public const B = 'b';
}

final class Printer {
    /** @param Kind::* $kind */
    public function interpolate(string $kind): string {
        return "kind: $kind";
    }

    /** @param Kind::* $kind */
    public function concat(string $kind): string {
        return $kind . '!';
    }

    /** @param Kind::* $kind */
    public function show(string $kind): void {
        echo $kind;
    }
}
