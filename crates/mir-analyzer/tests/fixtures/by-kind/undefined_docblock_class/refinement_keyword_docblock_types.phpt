===description===
Refinement keywords in class property `@var` docblocks without a native type
hint (`int-mask`, `non-empty-associative-array`, `class-string-map`) resolve
as keywords, never as nonexistent classes.
===config===
suppress=MissingPropertyType
===file===
<?php
class Rec {
    /** @var int-mask */
    private $mask;

    /** @var non-empty-associative-array */
    private $map;

    /** @var class-string-map */
    private $strings;

    /** @var non-empty-list */
    public array $items = [];
}

===expect===
