===description===
No ImplicitToStringCast when arg implements \Stringable — that interface signals intentional string use
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

// A class with __toString but NOT implementing \Stringable still warns
class HasToString {
    public function __toString(): string { return 'x'; }
}
process(new HasToString());
===expect===
ImplicitToStringCast@18:9-18:24: Class HasToString is implicitly cast to string
