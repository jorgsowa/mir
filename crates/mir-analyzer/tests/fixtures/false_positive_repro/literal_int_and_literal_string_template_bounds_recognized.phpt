===description===
`literal-int`/`literal-string` weren't recognized as docblock keywords —
they fell through to the named-class fallback, producing a bogus
`\literal-string`/`\literal-int`-named class as the template's bound. A
literal value passed as the bound-checked argument then always failed
the (nonsensical) bound.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @template A of literal-int|literal-string
 */
final class Box {
    /** @param A $v */
    public function __construct(int|string $v) {}
}
new Box('create');
new Box(42);
===expect===
