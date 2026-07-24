===description===
`$this->items['id'] = $tainted;` — the array-element-write taint arm
only ever matched a plain-variable base (`$arr['k'] = ...`), silently
dropping taint for a property-rooted array, even though the read side
already resolves taint through a property chain.
===config===
suppress=MissingPropertyType,MixedArrayAccess,MixedAssignment,MissingConstructor
===file===
<?php
class Cache {
    public $items = [];
}
function test(): void {
    $c = new Cache();
    $c->items['id'] = $_GET['id'];
    echo $c->items['id'];
}
===expect===
TaintedHtml@8:4-8:25: Tainted HTML output — possible XSS
