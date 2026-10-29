===description===
A ternary branch its condition rules out never runs, so it reports nothing; a branch that can run is still checked.
===file===
<?php
interface Htmlable { public function toHtml(): string; }
final class Page { public function title(): string { return ''; } }

function thenRuledOut(string $v): string {
    return $v instanceof Htmlable ? $v->toHtml() : $v;
}

function elseRuledOut(Page $p): string {
    return $p instanceof Page ? $p->title() : $p->toHtml();
}

function scalarCheck(string $v): string {
    return is_int($v) ? $v->format() : $v;
}

function nested(string $v, bool $b): string {
    return $b ? ($v instanceof Htmlable ? $v->toHtml() : $v) : $v;
}

function bothRun(string|Htmlable $v): string {
    return $v instanceof Htmlable ? $v->toHtml() : $v->toHtml();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'string'
//                                                 ^^^^^^^^^^^^ InvalidMethodCall: Cannot call method toHtml() on non-object type 'string'
}
