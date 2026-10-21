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
}

function firstClassCallableFallsBackToGlobal(): void {
    echo implode(',', array_map(intval(...), $_GET['l']));
    echo implode(',', array_map(htmlspecialchars(...), $_GET['l']));
    echo implode(',', array_map(trim(...), $_GET['l']));
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
}

function callbackSinkInNamespacedFunction(): void {
    array_map(show(...), $_GET['l']);
    array_map('App\Http\show', $_GET['l']);
}

function unrelatedCallbackWithCleanData(): void {
    array_map(show(...), ['a', 'b']);
}

function filterWithCallback(): void {
    echo implode(',', array_filter($_GET['l'], 'strlen'));
}
===expect===
TaintedHtml@19:4-19:55: Tainted HTML output — possible XSS
TaintedHtml@25:4-25:56: Tainted HTML output — possible XSS
TaintedHtml@44:4-44:55: Tainted HTML output — possible XSS
TaintedHtml@48:4-48:36: Tainted HTML output — possible XSS
TaintedHtml@49:4-49:42: Tainted HTML output — possible XSS
TaintedHtml@57:4-57:58: Tainted HTML output — possible XSS
