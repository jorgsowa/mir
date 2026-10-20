===description===
Pre-8 code guards DOMDocument factory results against false (now stripped from the type). Those guards stay unflagged, while an unrelated always-object method still flags a false comparison.
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function strictFalse(DOMDocument $document, string $name): ?DOMElement {
    $element = $document->createElement($name);
    if ($element === false) {
        return null;
    }
    return $element;
}

function strictNotFalse(DOMDocument $document, string $name): ?DOMElement {
    $element = $document->createElement($name);
    if ($element !== false) {
        return $element;
    }
    return null;
}

function falsy(DOMDocument $document, string $name): ?DOMElement {
    $element = $document->createElement($name);
    if (!$element) {
        return null;
    }
    $element->setAttribute('a', 'b');
    return $element;
}

function otherFactories(DOMDocument $document, string $name): void {
    if ($document->createAttribute($name) === false) {
        return;
    }
    if (false === $document->createElementNS('urn:example', $name)) {
        return;
    }
    if (!$document->createCDATASection($name)) {
        return;
    }
}

function genuinelyImpossible(DOMDocument $document, string $name): void {
    $text = $document->createTextNode($name);
    if ($text === false) {
//      ^^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'DOMText' and 'false' is always false — these types can never be identical
        return;
    }
}
