===description===
A string callback is a fully qualified name and a first-class callable resolves
like a call, falling back to the global function, so inside a namespace
`array_map('intval', ..)`, `array_map(intval(...), ..)` and namespaced user
functions are still known to return numbers, escape, or reach a sink.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App\Http;

function escaped(string $s): string {
    return htmlspecialchars($s);
}

function raw(string $s): string {
    return $s;
}

function show(string $s): void {
    echo $s;
}

function stringCallbackIsGlobal(): void {
    echo implode(',', array_map('intval', $_GET['l']));
    echo implode(',', array_map('htmlspecialchars', $_GET['l']));
    echo implode(',', array_map('strval', $_GET['l']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function firstClassCallableFallsBackToGlobal(): void {
    echo implode(',', array_map(intval(...), $_GET['l']));
    echo implode(',', array_map(htmlspecialchars(...), $_GET['l']));
    echo implode(',', array_map(trim(...), $_GET['l']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function viaExplode(): void {
    $ids = array_map('intval', explode(',', $_GET['ids']));
    echo implode(',', $ids);
}

function throughVariable(): void {
    $l = $_GET['l'];
    $ids = array_map('intval', $l);
    foreach ($ids as $id) {
        echo $id;
    }
}

function namespacedUserFunctions(): void {
    echo implode(',', array_map(escaped(...), $_GET['l']));
    echo implode(',', array_map('App\Http\escaped', $_GET['l']));
    echo implode(',', array_map(raw(...), $_GET['l']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function callbackSinkInNamespacedFunction(): void {
    array_map(show(...), $_GET['l']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    array_map('App\Http\show', $_GET['l']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function unrelatedCallbackWithCleanData(): void {
    array_map(show(...), ['a', 'b']);
}

function filterWithCallback(): void {
    echo implode(',', array_filter($_GET['l'], 'strlen'));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
