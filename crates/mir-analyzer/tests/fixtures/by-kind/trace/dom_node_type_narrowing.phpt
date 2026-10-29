===description===
Comparing a `DOMNode` variable's `nodeType` with an `XML_*_NODE` constant (or its value) narrows it to the class PHP creates for that node type; other comparisons and values are left alone.
===file===
<?php
function identical(\DOMNode $n): void {
    if ($n->nodeType === XML_ELEMENT_NODE) {
        /** @trace $n */
        echo $n->getAttribute('id');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Trace: Type of $n is DOMElement
    } else {
        /** @trace $n */
        echo 1;
//      ^^^^^^^ Trace: Type of $n is DOMNode
    }
}

function reversedAndQualified(\DOMNode $n): void {
    if (\XML_TEXT_NODE === $n->nodeType) {
        /** @trace $n */
        echo $n->wholeText;
//      ^^^^^^^^^^^^^^^^^^^ Trace: Type of $n is DOMText
    }
}

function loose(\DOMNode $n): void {
    if ($n->nodeType == XML_COMMENT_NODE) {
        /** @trace $n */
        echo 1;
//      ^^^^^^^ Trace: Type of $n is DOMComment
    }
}

function intValue(\DOMNode $n): void {
    if ($n->nodeType === 4) {
        /** @trace $n */
        echo 1;
//      ^^^^^^^ Trace: Type of $n is DOMCdataSection
    }
}

function notIdenticalThenSkip(\DOMNode $n): void {
    if (XML_ELEMENT_NODE !== $n->nodeType) {
        return;
    }
    /** @trace $n */
    echo 1;
//  ^^^^^^^ Trace: Type of $n is DOMElement
}

function nullable(?\DOMNode $n): void {
    if ($n->nodeType === XML_DOCUMENT_NODE) {
//      ^^^^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $nodeType on possibly null value
        /** @trace $n */
        echo 1;
//      ^^^^^^^ Trace: Type of $n is DOMDocument
    }
}

function unknownValue(\DOMNode $n): void {
    if ($n->nodeType === 6) {
        /** @trace $n */
        echo 1;
//      ^^^^^^^ Trace: Type of $n is DOMNode
    }
}

function otherProperty(\DOMNode $n): void {
    if ($n->nodeName === XML_ELEMENT_NODE) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and '1' is always false — these types can never be identical
        /** @trace $n */
        echo 1;
//      ^^^^^^^ Trace: Type of $n is DOMNode
    }
}

final class Holder { public int $nodeType = 1; }

function notDom(Holder $h): void {
    if ($h->nodeType === XML_ELEMENT_NODE) {
        /** @trace $h */
        echo 1;
//      ^^^^^^^ Trace: Type of $h is Holder
    }
}
