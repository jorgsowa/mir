===description===
The never-false createElement() result holds through a DOMDocument subclass, $this, nested calls and method chaining on the result.
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

class XmlBuilder extends DOMDocument {
    public function node(string $name): DOMElement {
        $element = $this->createElement($name);
        /** @mir-check $element is DOMElement */
        $element->setAttribute('name', $name);
        $this->appendChild($element);
        return $element;
    }

    public function viaFirstClassCallable(string $name): DOMElement {
        $factory = $this->createElement(...);
        return $factory($name);
    }
}

function nested(XmlBuilder $builder, string $outer, string $inner): DOMElement {
    $parent = $builder->createElement($outer);
    $parent->appendChild($builder->createElement($inner))->setAttribute('k', 'v');
    return $parent;
}

function ternary(DOMDocument $document, ?string $name): DOMElement {
    return $document->createElement($name ?? 'default');
}
