===description===
DOMDocument::createElement() explains that its result must be checked for false before insertion
===config===
suppress=MissingThrowsDocblock
===file===
<?php

$document = new DOMDocument();
$document->appendChild($document->createElement('root'));
===expect===
InvalidTemplateParam@4:0-4:56: Template type 'TNode' inferred as 'DOMElement|false' can be false and therefore does not satisfy bound 'DOMNode'
