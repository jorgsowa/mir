===description===
DOMElement::$attributes is a non-null DOMNamedNodeMap<DOMAttr>; only the inherited
DOMNode::$attributes (and other node kinds) may be null.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function elementAttributes(DOMElement $element): ?DOMAttr {
    /** @mir-check $element->attributes is DOMNamedNodeMap<DOMAttr> */
    $count = $element->attributes->length;
    $total = count($element->attributes);
    foreach ($element->attributes as $name => $attr) {
        /** @mir-check $attr is DOMAttr */
        echo $name, $attr->value;
    }
    return $element->attributes->getNamedItem('id');
}

function documentRoot(DOMDocument $document): int {
    $root = $document->documentElement;
    if ($root === null) {
        return 0;
    }
    return $root->attributes->length;
}

class XmlElement extends DOMElement {
    public function attributeCount(): int {
        return $this->attributes->length;
    }
}

function otherNodes(DOMNode $node, DOMAttr $attr, DOMText $text): void {
    $nodeAttributes = $node->attributes;
    /** @mir-check $nodeAttributes is DOMNamedNodeMap<DOMAttr>|null */
    $attrAttributes = $attr->attributes;
    /** @mir-check $attrAttributes is DOMNamedNodeMap<DOMAttr>|null */
    $textAttributes = $text->attributes;
    /** @mir-check $textAttributes is DOMNamedNodeMap<DOMAttr>|null */
    $length = $node->attributes->length;
//            ^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $length on possibly null value
    if ($node instanceof DOMElement) {
        $narrowed = $node->attributes->length;
    }
}
