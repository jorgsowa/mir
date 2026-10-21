===description===
An object holding tainted data keeps it when it moves into an array, a
foreach source, a closure capture or a call argument. A clean object, an
escaped property or a property no one reads stays unreported.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedPropertyFetch errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class W {
    public string $n = '';
    public string $m = 'fixed';

    public function __construct(string $n) {
        $this->n = $n;
    }
}

function show(W $w): void {
    echo $w->n;
}

function show_either(?W $w): void {
    if ($w !== null) {
        echo $w->n;
    }
}

class Printer {
    public function show(W $w): void {
        echo $w->n;
    }

    public static function showStatic(W $w): void {
        echo $w->n;
    }
}

function foreach_over_inline_new(): void {
    foreach ([new W($_GET['a'])] as $o) {
        echo $o->n;
//      ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function closure_capture(): void {
    $o = new W($_GET['a']);
    $f = function () use ($o): void {
        echo $o->n;
//      ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    };
    $f();
}

function closure_return(): void {
    $o = new W($_GET['a']);
    $f = function () use ($o): string {
        return $o->n;
    };
    echo $f();
//  ^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function list_literal(): void {
    $o = new W($_GET['a']);
    $l = [$o];
    echo $l[0]->n;
//  ^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function keyed_literal(): void {
    $o = new W($_GET['a']);
    $l = ['k' => $o];
    echo $l['k']->n;
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function pushed(): void {
    $o = new W($_GET['a']);
    $l = [];
    $l[] = $o;
    echo $l[0]->n;
//  ^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function keyed_write(): void {
    $o = new W($_GET['a']);
    $l = [];
    $l['k'] = $o;
    echo $l['k']->n;
//  ^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function setter_taints_then_boxed(): void {
    $o = new W('clean');
    $o->n = $_GET['a'];
    $l = [$o];
    echo $l[0]->n;
//  ^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function foreach_over_variable_list(): void {
    $l = [new W($_GET['a'])];
    foreach ($l as $o) {
        echo $o->n;
//      ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function passed_to_function(): void {
    $o = new W($_GET['a']);
    show($o);
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function passed_inline_new(): void {
    show(new W($_GET['a']));
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function passed_to_method(): void {
    $o = new W($_GET['a']);
    $p = new Printer();
    $p->show($o);
//  ^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function passed_to_static_method(): void {
    $o = new W($_GET['a']);
    Printer::showStatic($o);
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function passed_through_ternary(bool $flag): void {
    $o = new W($_GET['a']);
    show($flag ? $o : new W('clean'));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function passed_through_coalesce(?W $maybe): void {
    $o = new W($_GET['a']);
    show($maybe ?? $o);
//  ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function passed_in_array(): void {
    $o = new W($_GET['a']);
    show_all([$o]);
//  ^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function show_all(array $items): void {
    foreach ($items as $item) {
        echo $item->n;
    }
}

function clean_object_in_list(): void {
    $o = new W('clean');
    $l = [$o];
    echo $l[0]->n;
}

function clean_object_in_closure(): void {
    $o = new W('clean');
    $f = function () use ($o): void {
        echo $o->n;
    };
    $f();
}

function clean_object_passed(): void {
    show(new W('clean'));
    $o = new W('clean');
    show($o);
    (new Printer())->show($o);
}

function escaped_property_in_list(): void {
    $o = new W(htmlspecialchars($_GET['a']));
    $l = [$o];
    echo $l[0]->n;
    show($o);
}

function overwritten_before_boxing(): void {
    $o = new W($_GET['a']);
    $o->n = 'clean';
    $l = [$o];
    echo $l[0]->n;
    show($o);
}

function rebuilt_before_passing(): void {
    $o = new W($_GET['a']);
    echo $o->m;
    $o = new W('clean');
    show($o);
}

function builtin_on_tainted_object_is_not_a_leak(): void {
    $o = new W($_GET['a']);
    echo get_class($o);
    echo spl_object_id($o);
}
