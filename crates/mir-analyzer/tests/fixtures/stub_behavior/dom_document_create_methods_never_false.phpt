===description===
DOMDocument factory methods that throw DOMException on invalid input in PHP 8 never return false, so a dynamic name can be appended without a false guard.
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function build(DOMDocument $document, DOMElement $root, string $name): void {
    $root->appendChild($document->createElement($name));
    $root->appendChild($document->createElement($name, 'value'));
    $root->appendChild($document->createElementNS('urn:example', $name));
    $root->appendChild($document->createAttribute($name));
    $root->appendChild($document->createCDATASection($name));
    $root->appendChild($document->createProcessingInstruction($name, 'data'));
    $root->appendChild($document->createEntityReference($name));
    $root->appendChild($document->createTextNode($name));
    $root->appendChild($document->createComment($name));
}

function types(DOMDocument $document, string $name): void {
    $element = $document->createElement($name);
    /** @mir-check $element is DOMElement */
    $elementNs = $document->createElementNS('urn:example', $name);
    /** @mir-check $elementNs is DOMElement */
    $attribute = $document->createAttribute($name);
    /** @mir-check $attribute is DOMAttr */
    $cdata = $document->createCDATASection($name);
    /** @mir-check $cdata is DOMCDATASection */
    $instruction = $document->createProcessingInstruction($name);
    /** @mir-check $instruction is DOMProcessingInstruction */
    $reference = $document->createEntityReference($name);
    /** @mir-check $reference is DOMEntityReference */
}
