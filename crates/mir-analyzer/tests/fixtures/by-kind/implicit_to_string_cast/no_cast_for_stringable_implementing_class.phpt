===description===
No ImplicitToStringCast when class implements \Stringable — and also no warning for __toString-only classes
===config===
suppress=UnusedParam
===file===
<?php
class FluentString implements \Stringable {
    public function __toString(): string { return 'hello'; }
}

/**
 * @param string $value
 */
function process(string $value): void {}

process(new FluentString());

// A class with __toString but NOT implementing \Stringable is also fine —
// __toString is sufficient for PHP's coercive string conversion.
class HasToString {
    public function __toString(): string { return 'x'; }
}
process(new HasToString());
===expect===
