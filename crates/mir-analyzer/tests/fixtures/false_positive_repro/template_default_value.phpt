===description===
`@template T = Default` (a template parameter's default value, used when
nothing binds T) was silently discarded: parse_template_line only
recognized `of`/`as` after the name, so the ` = string` trailer was left
attached to (and corrupting) the bound parse, and TemplateParam had no
field to store it even if parsed. An unbound template then always fell
back to `mixed` instead of its declared default.
===file===
<?php
/**
 * @template T = string
 * @return T
 */
function make() {
    return "fallback";
}
$x = make();
/** @mir-check $x is string */
echo $x;

final class Name implements \Stringable {
    public function __toString(): string {
        return "anonymous";
    }
}

/**
 * @template T of \Stringable = Name
 * @return T
 */
function makeWithBound() {
    return new Name();
}
$y = makeWithBound();
/** @mir-check $y is Name */
echo (string) $y;
===expect===
