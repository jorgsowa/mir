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
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function arrayObjectIterate() {
    $o = new ArrayObject($_GET);
    foreach ($o as $v) {
        echo $v;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function arrayObjectKey() {
    $o = new ArrayObject($_GET);
    foreach ($o as $k => $v) {
        echo $k;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function arrayObjectCopy() {
    $o = new ArrayObject($_GET);
    foreach ($o->getArrayCopy() as $v) {
        echo $v;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function arrayObjectGet() {
    $o = new ArrayObject($_GET);
    echo $o->offsetGet('a');
//  ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function arrayIteratorCurrent() {
    $it = new ArrayIterator($_GET);
    echo $it->current();
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function arrayIteratorIterate() {
    foreach (new ArrayIterator($_GET) as $v) {
        echo $v;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function arrayIteratorToArray() {
    foreach (iterator_to_array(new ArrayIterator($_GET)) as $v) {
        echo $v;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function nestedIterators() {
    $it = new RecursiveIteratorIterator(new RecursiveArrayIterator($_GET));
    foreach ($it as $v) {
        echo $v;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function limitedIterator() {
    foreach (new LimitIterator(new ArrayIterator($_GET), 0, 2) as $v) {
        echo $v;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function subclass() {
    $o = new Bag($_GET);
    echo $o['a'];
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function appendedData() {
    $o = new ArrayObject();
    $o->append($_GET['a']);
    foreach ($o as $v) {
        echo $v;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function exchangedData() {
    $o = new ArrayObject();
    $o->exchangeArray($_GET);
    echo $o['a'];
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function assignedOffset() {
    $o = new ArrayObject();
    $o['k'] = $_GET['a'];
    echo $o['k'];
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function fromTaintedVariable() {
    $items = $_GET['items'];
    $o = new ArrayObject($items);
    echo $o['a'];
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function fromParameter(array $items) {
    return new ArrayObject($items);
}

function throughFunction() {
    $o = fromParameter($_GET);
    echo $o['a'];
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
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
