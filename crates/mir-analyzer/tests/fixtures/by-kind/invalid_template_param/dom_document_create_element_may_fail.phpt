===description===
DOMDocument::createElement() explains that a dynamically named result must be checked for false before insertion
===config===
suppress=MissingThrowsDocblock
===file===
<?php

function elementName(): string {
    return 'root';
}

$document = new DOMDocument();
$document->appendChild($document->createElement(elementName()));
===expect===
InvalidTemplateParam@8:0-8:63: Template type 'TNode' inferred as 'DOMElement|false' can be false and therefore does not satisfy bound 'DOMNode'
