===description===
DOMNode-pattern: global class used as template bound from a namespaced file (without leading slash)
===file===
<?php
namespace App;

/**
 * @template T of DOMNode
 * @param T $node
 */
function process($node): void {
    $node;
}

process(new \DOMElement('x'));
===expect===
