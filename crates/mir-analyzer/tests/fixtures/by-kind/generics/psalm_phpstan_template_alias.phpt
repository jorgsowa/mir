===description===
`@psalm-template`/`@phpstan-template` (and their -covariant/-contravariant
forms) must be recognized as aliases for `@template`, matching the existing
psalm-/phpstan- alias support for `@param`, `@return`, `@assert`, etc.
Libraries authored against Psalm/PHPStan specifically often use the
prefixed form instead of the bare PHPDoc standard tag.
===config===
suppress=MissingReturnType,UnusedParam,UnusedVariable
===file===
<?php
/**
 * @psalm-template T
 */
class Box {
    /** @param T $x */
    public function __construct($x) {}
}

/**
 * @phpstan-template T
 */
class OtherBox {
    /** @param T $x */
    public function __construct($x) {}
}

$box = new Box(1);
/** @mir-check $box is Box<int> */
echo "ok";

$other = new OtherBox("s");
/** @mir-check $other is OtherBox<string> */
echo "ok";
===expect===
