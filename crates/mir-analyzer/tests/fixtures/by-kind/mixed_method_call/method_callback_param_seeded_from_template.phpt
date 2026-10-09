===description===
An untyped closure param takes the type the callee's `callable(T)` passes, bound through the receiver's type args.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Item {
    public function name(): string { return 'x'; }
}

/** @template T */
final class Box {
    /** @param callable(T): void $fn */
    public function each(callable $fn): void {}
}

/** @param Box<Item> $box */
function seeded(Box $box): void {
    $box->each(function ($i) {
        /** @mir-check $i is Item */
        $i->name();
    });
    $box->each(fn($i) => $i->name());
}

function unbound(Box $box): void {
    $box->each(function ($i) {
        /** @mir-check $i is mixed */
    });
}
