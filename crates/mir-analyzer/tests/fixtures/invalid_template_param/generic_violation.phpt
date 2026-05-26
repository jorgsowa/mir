===description===
Violation: type that doesn't satisfy bound is rejected
===file:test.php===
<?php
class Base {}
class Unrelated {}

/**
 * @template T of Base
 * @param T $item
 */
function process($item): void {
    echo get_class($item);
}

$unrelated = new Unrelated();
process($unrelated);
===expect===
test.php: InvalidTemplateParam@14:1: Template type 'T' inferred as 'Unrelated' does not satisfy bound 'Base'
