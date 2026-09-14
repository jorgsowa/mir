===description===
DOMNodeList is covariant: lists of elements are accepted where lists of nodes are required, including through a class template
===file===
<?php
/** @param DOMNodeList<DOMNode> $nodes */
function snapshot(DOMNodeList $nodes): void {
    $nodes->count();
}

/** @template T of DOMNode */
final class NodeSnapshot {
    /** @param DOMNodeList<T> $nodes */
    public function capture(DOMNodeList $nodes): void {
        $nodes->count();
    }
}

$document = new DOMDocument();
snapshot($document->getElementsByTagName('item'));

$element = new DOMElement('container');
snapshot($element->getElementsByTagName('item'));

/** @var NodeSnapshot<DOMNode> $allNodes */
$allNodes = new NodeSnapshot();
$allNodes->capture($document->getElementsByTagName('item'));
===expect===
