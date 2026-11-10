===description===
`switch (get_class($x))`, `switch ($x::class)` and `switch (gettype($x))`
narrow `$x` inside each case to the type its label names, including a body
reached from several stacked labels.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Foo { public function fooOnly(): string { return 'foo'; } }
final class Bar { public function barOnly(): string { return 'bar'; } }
final class Baz {}

function byGetClass(Foo|Bar $x): void {
    switch (get_class($x)) {
        case Foo::class:
            echo $x->fooOnly();
            break;
        case Bar::class:
            echo $x->barOnly();
            break;
    }
}

function byClassConst(Foo|Bar|Baz $x): void {
    switch ($x::class) {
        case Foo::class:
        case Bar::class:
            /** @mir-check $x is Foo|Bar */
            $_ = $x;
            break;
    }
}

function byGettype(int|string $x): void {
    switch (gettype($x)) {
        case 'integer':
            /** @mir-check $x is int */
            $_ = $x;
            break;
    }
}
