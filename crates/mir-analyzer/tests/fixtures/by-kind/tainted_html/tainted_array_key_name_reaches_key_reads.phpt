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
    }
}

function keyFromVariable() {
    $key = $_GET['k'];
    $a = [];
    $a[$key] = 1;
    foreach ($a as $k => $_) {
        echo $k;
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
    }
}

function compoundWrite() {
    $a = [];
    $a[$_GET['k']] = 0;
    $a[$_GET['k']] += 1;
    foreach ($a as $k => $_) {
        echo $k;
    }
}

function keysFunction() {
    $a = [];
    $a[$_GET['k']] = 1;
    foreach (array_keys($a) as $k) {
        echo $k;
    }
}

function firstKey() {
    $a = [];
    $a[$_GET['k']] = 1;
    echo array_key_first($a);
}

function flipped() {
    $a = [];
    $a[$_GET['k']] = 1;
    foreach (array_flip($a) as $k => $v) {
        echo $v;
    }
}

function serialized() {
    $a = [];
    $a[$_GET['k']] = 1;
    echo json_encode($a);
}

function copied() {
    $a = [];
    $a[$_GET['k']] = 1;
    $b = $a;
    foreach ($b as $k => $_) {
        echo $k;
    }
}

function viaBranch($c) {
    $a = [];
    if ($c) {
        $a[$_GET['k']] = 1;
    }
    foreach ($a as $k => $_) {
        echo $k;
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
===expect===
TaintedHtml@6:8-6:16: Tainted HTML output — possible XSS
TaintedHtml@15:8-15:16: Tainted HTML output — possible XSS
TaintedHtml@37:8-37:16: Tainted HTML output — possible XSS
TaintedHtml@46:8-46:16: Tainted HTML output — possible XSS
TaintedHtml@54:8-54:16: Tainted HTML output — possible XSS
TaintedHtml@61:4-61:29: Tainted HTML output — possible XSS
TaintedHtml@68:8-68:16: Tainted HTML output — possible XSS
TaintedHtml@75:4-75:25: Tainted HTML output — possible XSS
TaintedHtml@83:8-83:16: Tainted HTML output — possible XSS
TaintedHtml@93:8-93:16: Tainted HTML output — possible XSS
