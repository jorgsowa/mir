===description===
Refinement docblock keywords are not resolved as classes.
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
