===description===
DOMDocument::createElement() returns DOMElement, satisfying DOM insertion template bounds
===config===
suppress=MissingThrowsDocblock
===file===
<?php

$document = new DOMDocument();
$root = $document->createElement('root');

$document->appendChild($root);
$root->appendChild($document->createElement('child', 'value'));
$root->insertBefore($document->createElement('first'), $root->firstChild);
===expect===
