===description===
A promoted constructor property's docblock literal-union refinement
(`@param 'a'|'b' $value` alongside a native `string` hint) must attach to
the property — previously only a plain unspecialized `array`/`mixed`
native hint let the docblock through at all; any other concrete native
scalar hint discarded the docblock refinement entirely.
===config===
suppress=UnusedParam
===file===
<?php
final class Attr {
    /** @param 'a'|'b' $value */
    public function __construct(public string $value) {}
}
function f(Attr $attr): int {
    return match ($attr->value) { 'a' => 1, 'b' => 2 };
}
===expect===
