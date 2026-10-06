===description===
`DOMXPath::query` yields `DOMNodeList<DOMNode>`, so its nodes can be passed where a `DOMNode` is required.
===file===
<?php
function removeMatches(DOMXPath $xpath, DOMNode $parent): void {
    $nodes = $xpath->query('//item');
    if ($nodes === false) {
        return;
    }
    /** @mir-check $nodes is DOMNodeList<DOMNode> */
    foreach ($nodes as $node) {
        /** @mir-check $node is DOMNode */
        $parent->removeChild($node);
    }
}

function firstMatch(DOMXPath $xpath, DOMNode $parent): void {
    $nodes = $xpath->query('//item', $parent);
    if ($nodes !== false) {
        /** @mir-check $nodes is DOMNodeList<DOMNode> */
        echo $nodes->length;
    }
}
