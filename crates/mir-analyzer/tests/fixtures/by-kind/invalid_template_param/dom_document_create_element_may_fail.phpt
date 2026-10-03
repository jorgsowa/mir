===description===
DOMDocument::createElement() explains that a dynamically named result must be checked for false before insertion
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function elementName(): string {
    return 'root';
}

$document = new DOMDocument();
$document->appendChild($document->createElement(elementName()));
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'TNode' inferred as 'DOMElement|false' can be false and therefore does not satisfy bound 'DOMNode'
===expect===
