===description===
A closure param declared as a union keeps every member when the callee only
passes one bound type, so `instanceof` on another member stays meaningful.
Single and nullable declared types are still narrowed to the bound type.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {}
final class Ok extends Base {}
final class Fail extends Base { public function reason(): string { return 'x'; } }

function takesFail(Fail $f): string { return $f->reason(); }

/** @template E */
class Box {
    /** @param callable(E): mixed $f */
    public function each(callable $f): void {}
}

/** @return Box<Ok> */
function box(): Box { return new Box(); }

box()->each(function (Ok|Fail $r): void {
    /** @mir-check $r is Ok|Fail */
    $_ = $r;
    if ($r instanceof Fail) {
        takesFail($r);
    }
});

box()->each(fn(Ok|Fail $r) => $r instanceof Fail ? takesFail($r) : null);

box()->each(function (?Ok $r): void {
    /** @mir-check $r is Ok */
    $_ = $r;
});

box()->each(function (Base $r): void {
    /** @mir-check $r is Ok */
    $_ = $r;
});
