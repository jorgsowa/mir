===description===
`ArrayObject`, `ArrayIterator` and the SPL iterator wrappers hand out the data
they were built from or that is stored in them: reads, iteration and copies of a
container built from request data carry its taint. Clean data stays unreported.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <UnusedForeachValue errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Bag extends ArrayObject {
}

class Own extends ArrayObject {
    public function __construct(array $items) {
        parent::__construct([]);
    }
}

function arrayObjectRead() {
    $o = new ArrayObject($_GET);
    echo $o['a'];
}

function arrayObjectIterate() {
    $o = new ArrayObject($_GET);
    foreach ($o as $v) {
        echo $v;
    }
}

function arrayObjectKey() {
    $o = new ArrayObject($_GET);
    foreach ($o as $k => $v) {
        echo $k;
    }
}

function arrayObjectCopy() {
    $o = new ArrayObject($_GET);
    foreach ($o->getArrayCopy() as $v) {
        echo $v;
    }
}

function arrayObjectGet() {
    $o = new ArrayObject($_GET);
    echo $o->offsetGet('a');
}

function arrayIteratorCurrent() {
    $it = new ArrayIterator($_GET);
    echo $it->current();
}

function arrayIteratorIterate() {
    foreach (new ArrayIterator($_GET) as $v) {
        echo $v;
    }
}

function arrayIteratorToArray() {
    foreach (iterator_to_array(new ArrayIterator($_GET)) as $v) {
        echo $v;
    }
}

function nestedIterators() {
    $it = new RecursiveIteratorIterator(new RecursiveArrayIterator($_GET));
    foreach ($it as $v) {
        echo $v;
    }
}

function limitedIterator() {
    foreach (new LimitIterator(new ArrayIterator($_GET), 0, 2) as $v) {
        echo $v;
    }
}

function subclass() {
    $o = new Bag($_GET);
    echo $o['a'];
}

function appendedData() {
    $o = new ArrayObject();
    $o->append($_GET['a']);
    foreach ($o as $v) {
        echo $v;
    }
}

function exchangedData() {
    $o = new ArrayObject();
    $o->exchangeArray($_GET);
    echo $o['a'];
}

function assignedOffset() {
    $o = new ArrayObject();
    $o['k'] = $_GET['a'];
    echo $o['k'];
}

function fromTaintedVariable() {
    $items = $_GET['items'];
    $o = new ArrayObject($items);
    echo $o['a'];
}

function fromParameter(array $items) {
    return new ArrayObject($items);
}

function throughFunction() {
    $o = fromParameter($_GET);
    echo $o['a'];
}

function cleanArray() {
    $o = new ArrayObject(['a' => 'literal']);
    echo $o['a'];
}

function cleanIterate() {
    foreach (new ArrayIterator(['literal']) as $v) {
        echo $v;
    }
}

function emptyConstruction() {
    $o = new ArrayObject();
    echo $o['a'];
}

function overriddenConstructor() {
    $o = new Own($_GET);
    echo $o['a'];
}

function sanitizedRead() {
    $o = new ArrayObject($_GET);
    echo htmlspecialchars($o['a']);
}

function replacedContainer() {
    $o = new ArrayObject($_GET);
    $o = new ArrayObject(['a' => 'literal']);
    echo $o['a'];
}

function countOnly() {
    $o = new ArrayObject($_GET);
    echo count($o);
}
===expect===
TaintedHtml@13:4-13:17: Tainted HTML output — possible XSS
TaintedHtml@19:8-19:16: Tainted HTML output — possible XSS
TaintedHtml@26:8-26:16: Tainted HTML output — possible XSS
TaintedHtml@33:8-33:16: Tainted HTML output — possible XSS
TaintedHtml@39:4-39:28: Tainted HTML output — possible XSS
TaintedHtml@44:4-44:24: Tainted HTML output — possible XSS
TaintedHtml@49:8-49:16: Tainted HTML output — possible XSS
TaintedHtml@55:8-55:16: Tainted HTML output — possible XSS
TaintedHtml@62:8-62:16: Tainted HTML output — possible XSS
TaintedHtml@68:8-68:16: Tainted HTML output — possible XSS
TaintedHtml@74:4-74:17: Tainted HTML output — possible XSS
TaintedHtml@81:8-81:16: Tainted HTML output — possible XSS
TaintedHtml@88:4-88:17: Tainted HTML output — possible XSS
TaintedHtml@94:4-94:17: Tainted HTML output — possible XSS
TaintedHtml@100:4-100:17: Tainted HTML output — possible XSS
TaintedHtml@109:4-109:17: Tainted HTML output — possible XSS
