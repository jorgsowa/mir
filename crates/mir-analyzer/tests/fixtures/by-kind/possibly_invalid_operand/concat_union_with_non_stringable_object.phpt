===description===
Concatenating a union where only some members are objects without `__toString()` is PossiblyInvalidOperand; an operand that is always such an object stays ImplicitToStringCast.
===file===
<?php
final class GeneratedUrl {}

/** @return ($collect is true ? GeneratedUrl : string) */
function url(bool $collect = false): string|GeneratedUrl {
    return $collect ? new GeneratedUrl() : '/';
}

function partial(bool $collect): string {
    return 'URL=' . url($collect);
//         ^^^^^^^^^^^^^^^^^^^^^^ PossiblyInvalidOperand: Operator '.' might not be supported between '"URL="' and 'GeneratedUrl|string'
}

function always(GeneratedUrl $url): string {
    return 'URL=' . $url;
//                  ^^^^ ImplicitToStringCast: Class GeneratedUrl is implicitly cast to string
}
