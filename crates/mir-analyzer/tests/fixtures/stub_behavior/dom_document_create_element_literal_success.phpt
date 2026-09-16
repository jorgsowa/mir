===description===
DOMDocument::createElement() returns DOMElement for valid literal XML names.
===config===
suppress=MissingThrowsDocblock,UnusedParam,UnusedVariable
===file===
<?php

function acceptsNode(DOMNode $node): void {}

function runtimeName(): string {
    return 'runtime';
}

$document = new DOMDocument();
$parent = $document->createElement('parent');

$parent->appendChild($document->createElement('report'));
acceptsNode($document->createElement('include'));

$named = $document->createElement(value: 'text', localName: 'ns:entry');
/** @mir-check $named is DOMElement */

$literalName = 'item-1';
$stored = $document->createElement($literalName);
/** @mir-check $stored is DOMElement */

$unicode = $document->createElement('élément');
/** @mir-check $unicode is DOMElement */

$invalid = $document->createElement('not a name');
/** @mir-check $invalid is DOMElement|false */

$dynamic = $document->createElement(runtimeName());
/** @mir-check $dynamic is DOMElement|false */
===expect===
