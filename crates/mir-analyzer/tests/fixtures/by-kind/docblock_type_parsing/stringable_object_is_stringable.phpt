===description===
A `stringable-object` parameter is a `Stringable` inside the body and takes declared and implicit implementations.
===file===
<?php
final class Label implements Stringable {
    public function __toString(): string {
        return 'label';
    }
}

final class Slug {
    public function __toString(): string {
        return 'slug';
    }
}

/** @param stringable-object $value */
function describe($value): string {
    /** @mir-check $value is Stringable */
    $text = $value->__toString();
    return $text . (string) $value;
}

function run(Stringable $any): void {
    echo describe(new Label());
    echo describe(new Slug());
    echo describe($any);
}
