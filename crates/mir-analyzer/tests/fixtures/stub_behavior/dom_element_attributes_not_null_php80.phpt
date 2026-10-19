===description===
Before PHP 8.1 the DOM properties are untyped, so the @var docblocks apply directly:
DOMElement::$attributes stays non-null and DOMNode::$attributes stays nullable.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php

function attributes(DOMElement $element, DOMNode $node): void {
    /** @mir-check $element->attributes is DOMNamedNodeMap<DOMAttr> */
    $item = $element->attributes->getNamedItem('id');
    $nodeAttributes = $node->attributes;
    /** @mir-check $nodeAttributes is DOMNamedNodeMap<DOMAttr>|null */
}
