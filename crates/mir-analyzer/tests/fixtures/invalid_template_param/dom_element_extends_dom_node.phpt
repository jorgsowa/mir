===description===
DOMElement satisfies template bound of DOMNode parent class
===file===
<?php
/**
 * @template TNode of DOMNode
 * @param TNode $node
 */
function process($node): void {
    echo get_class($node);
}

$el = new DOMElement('x');
process($el);
===expect===
