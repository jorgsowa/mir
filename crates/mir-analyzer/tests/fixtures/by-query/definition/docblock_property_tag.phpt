===description===
A `@property` member resolves to its tag, not the class declaration.
===cursor===
definition
===file===
<?php
/**
 * @property int $count
 * @property-read string $label
 */
class Bag {}
function run(Bag $b): string {
    return $b->la<CURSOR>bel;
}
===expect===
test.php@4:3-4:31
