===description===
DOMNodeList<DOMElement> can be passed across files to PHPUnit-style snapshot helpers that accept DOMNodeList<DOMNode>
===file:snapshot.php===
<?php
namespace Fixture;

use DOMNode;
use DOMNodeList;

final class SnapshotNodeList {
    /** @param DOMNodeList<DOMNode> $list */
    public static function fromNodeList(DOMNodeList $list): self {
        $list->count();

        return new self();
    }
}
===file:app.php===
<?php
namespace Fixture;

use DOMDocument;
use DOMElement;

$document = new DOMDocument();
$root = new DOMElement('root');

SnapshotNodeList::fromNodeList($document->getElementsByTagName('testcase'));
SnapshotNodeList::fromNodeList($root->getElementsByTagName('testcase'));
===expect===
