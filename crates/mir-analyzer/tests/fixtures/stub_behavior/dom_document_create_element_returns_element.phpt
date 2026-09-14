===description===
DOMDocument::createElement() returns a usable DOMElement through assigned and generic flows
===config===
php_version=8.0
suppress=MissingThrowsDocblock
===file===
<?php

/**
 * @template TNode of DOMNode
 * @param TNode $node
 * @return TNode
 */
function append(DOMElement $parent, $node) {
    return $parent->appendChild($node);
}

$document = new DOMDocument();
$root = $document->createElement(localName: 'root');
$child = $document->createElement(localName: 'child', value: 'value');

$document->appendChild($root);
$attached = append($root, $child);
$attached->setAttribute('id', 'child');
===expect===
