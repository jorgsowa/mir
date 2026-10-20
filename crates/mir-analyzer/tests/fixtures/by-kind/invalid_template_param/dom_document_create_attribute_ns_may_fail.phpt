===description===
DOMDocument::createAttributeNS() really returns false (no root element), so it still must be checked before insertion.
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function insert(DOMDocument $document, DOMElement $root, string $name): void {
    $root->appendChild($document->createAttributeNS('urn:example', $name));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'TNode' inferred as 'DOMAttr|false' can be false and therefore does not satisfy bound 'DOMNode'
}

function guarded(DOMDocument $document, DOMElement $root, string $name): void {
    $attribute = $document->createAttributeNS('urn:example', $name);
    if ($attribute !== false) {
        $root->appendChild($attribute);
    }
}
