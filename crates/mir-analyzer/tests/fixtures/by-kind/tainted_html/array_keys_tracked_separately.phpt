===description===
Taint written to one literal key of an array doesn't spread to its other
keys. A dynamic key, an append or a merge of branches that disagree falls
back to the whole array. A value or property whose type can't carry a
payload isn't tainted.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <PossiblyUndefinedArrayOffset errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function literalKeys(string $name): void {
    $row = ['name' => $name, 'query' => $_GET['q']];
    echo $row['name'];
    echo $row['query'];
//  ^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $row['query'] = htmlspecialchars((string) $row['query']);
    echo $row['query'];
    $copy = $row;
    echo $copy['name'];
}

function writes(string $label): void {
    $view = [];
    $view['title'] = $label;
    $view['search'] = $_GET['q'];
    echo $view['title'];
    echo $view['search'];
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $view['meta']['author'] = $_GET['author'];
    echo $view['meta']['author'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo $view['title'];
    /** @mir-check $view is array{title: string, search: mixed, meta: array{author: mixed}} */
    echo count($view);
}

function coarse(string $label, string $key): void {
    $view = ['title' => $label];
    $view[$key] = $_GET['q'];
    echo $view['title'];
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $list = ['title' => $label];
    $list[] = $_GET['q'];
    echo $list['title'];
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function branches(string $label, bool $flag): void {
    $view = ['title' => $label];
    if ($flag) {
        $view['search'] = $_GET['q'];
    }
    echo $view['title'];
    echo $view['search'] ?? '';
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

final class Product {
    public int $id = 0;
    public string $name = '';
}

function typed(Product $product): void {
    $product->id = (int) $_GET['id'];
    $product->name = (string) $_GET['name'];
    echo $product->id;
    echo $product->name;
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $counts = ['total' => 3, 'query' => $_GET['q']];
    echo $counts['total'];
}
