===description===
A parameter a closure or arrow function captures keeps its link to the
enclosing function's caller: the function's summary records the sink or
return it reaches through the closure, so tainted arguments are reported at
the call. Escaping inside the closure still cleans it.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
    <MixedInferredReturnType errorLevel="suppress"/>
    <MissingClosureParamType errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function render(string $html): void {
    $show = function () use ($html) {
        echo $html;
    };
    $show();
}

function renderEach(array $items, string $suffix): void {
    array_walk($items, function ($item) use ($suffix) {
        echo $item . $suffix;
    });
}

function wrap(string $text): string {
    $wrap = fn () => '<b>' . $text . '</b>';
    return $wrap();
}

function prefixAll(array $items, string $prefix): array {
    return array_map(fn ($item) => $prefix . $item, $items);
}

function escaped(string $text): string {
    $escape = function () use ($text) {
        return htmlspecialchars($text);
    };
    return $escape();
}

function controller(): void {
    render($_GET['a']);
//  ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    render('<b>static</b>');
    renderEach(['x'], $_GET['s']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo wrap($_GET['w']);
//  ^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo implode(',', prefixAll(['x'], $_GET['p']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo escaped($_GET['e']);
}
