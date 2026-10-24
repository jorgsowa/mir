===description===
Writing under a tainted key (`$a[$_GET['k']] = 1`) taints the key names of the
array: iterating its keys, `array_keys()`, `array_key_first()`, `array_flip()`
and serializing the array carry that taint, while reading a value under another
key and iterating only the values do not.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <UnusedForeachValue errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
    <PossiblyUndefinedArrayOffset errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function keyOfForeach() {
    $a = [];
    $a[$_GET['k']] = 1;
    foreach ($a as $k => $_) {
        echo $k;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function keyFromVariable() {
    $key = $_GET['k'];
    $a = [];
    $a[$key] = 1;
    foreach ($a as $k => $_) {
        echo $k;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function valueOfForeach() {
    $a = [];
    $a[$_GET['k']] = 1;
    foreach ($a as $k => $v) {
        echo $v;
    }
}

function otherKeyRead() {
    $a = ['safe' => 'literal'];
    $a[$_GET['k']] = 1;
    echo $a['safe'];
}

function nestedWrite() {
    $a = [];
    $a[$_GET['k']][] = 1;
    foreach ($a as $k => $_) {
        echo $k;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function compoundWrite() {
    $a = [];
    $a[$_GET['k']] = 0;
    $a[$_GET['k']] += 1;
    foreach ($a as $k => $_) {
        echo $k;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function keysFunction() {
    $a = [];
    $a[$_GET['k']] = 1;
    foreach (array_keys($a) as $k) {
        echo $k;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function firstKey() {
    $a = [];
    $a[$_GET['k']] = 1;
    echo array_key_first($a);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function flipped() {
    $a = [];
    $a[$_GET['k']] = 1;
    foreach (array_flip($a) as $k => $v) {
        echo $v;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function serialized() {
    $a = [];
    $a[$_GET['k']] = 1;
    echo json_encode($a);
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function copied() {
    $a = [];
    $a[$_GET['k']] = 1;
    $b = $a;
    foreach ($b as $k => $_) {
        echo $k;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function viaBranch($c) {
    $a = [];
    if ($c) {
        $a[$_GET['k']] = 1;
    }
    foreach ($a as $k => $_) {
        echo $k;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function cleanKey() {
    $a = [];
    $a['literal'] = 1;
    foreach ($a as $k => $_) {
        echo $k;
    }
}

function integerKey() {
    $a = [];
    $a[(int) $_GET['k']] = 1;
    foreach ($a as $k => $_) {
        echo $k;
    }
}

function escapedKey() {
    $a = [];
    $a[htmlspecialchars($_GET['k'])] = 1;
    foreach ($a as $k => $_) {
        echo $k;
    }
}

function overwritten() {
    $a = [];
    $a[$_GET['k']] = 1;
    $a = [];
    foreach ($a as $k => $_) {
        echo $k;
    }
}

function valuesOnly() {
    $a = [];
    $a[$_GET['k']] = 'literal';
    foreach ($a as $v) {
        echo $v;
    }
}
