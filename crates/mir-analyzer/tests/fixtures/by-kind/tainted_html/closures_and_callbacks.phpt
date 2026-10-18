===description===
Closures and arrow functions are summarized like functions: calling one
passes on the taint its body returns, and an argument reaching a sink inside
it is reported at the call. Builtins that call back, like `array_map()`,
apply the callback's effect to the array's elements, so a sanitizing
callback cleans the result.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedFunctionCall errorLevel="suppress"/>
    <MissingClosureParamType errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedClosureParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function closures(): void {
    $identity = fn ($v) => $v;
    echo $identity($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $escape = fn ($v) => htmlspecialchars($v);
    echo $escape($_GET['x']);
    $fixed = function ($v) { return 'fixed'; };
    echo $fixed($_GET['x']);
    $render = function ($html) { echo $html; };
    $render($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $render('<b>static</b>');
    $alias = $identity;
    echo $alias($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (fn ($a, $b) => $b)('safe', $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo (fn ($a, $b) => $b)($_GET['x'], 'safe');
}

function callbacks(): void {
    echo implode(',', array_map('htmlspecialchars', $_GET['tags']));
    echo implode(',', array_map(htmlspecialchars(...), $_GET['tags']));
    echo implode(',', array_map(fn ($t) => htmlspecialchars($t), $_GET['tags']));
    $escaped = array_map(function ($t) { return htmlspecialchars($t); }, $_GET['tags']);
    echo implode(',', $escaped);
    echo implode(',', array_map('intval', $_GET['ids']));
    echo implode(',', array_map('trim', $_GET['tags']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo implode(',', array_map(fn ($t) => $t, $_GET['tags']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo implode(',', array_filter($_GET['tags'], 'strlen'));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    array_walk($_GET['tags'], function ($tag) { echo $tag; });
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    array_walk($_GET['tags'], function ($tag) { echo htmlspecialchars($tag); });
}

function show(string $html): void { echo $html; }

function namedCallbacks(): void {
    array_walk($_GET['tags'], 'show');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
