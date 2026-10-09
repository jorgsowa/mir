===description===
Closure params stay as written when the source is not a plain array, the element type is mixed, or the param is typed or by-reference.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Item {
    public function name(): string { return 'x'; }
}

/**
 * @param list<Item>|null $maybe
 * @param list<Item> $items
 */
function unsafe(array $untyped, ?array $maybe, array $items): void {
    array_map(fn($i) => $i->name(), $untyped);
//                      ^^^^^^^^^^ MixedMethodCall: Method name() called on mixed type
    array_filter($maybe ?? [], fn($i) => $i->name() !== '');

    array_map(function (mixed $i) {
        /** @mir-check $i is mixed */
        return 1;
    }, $items);

    array_map(function (object $i) {
        /** @mir-check $i is Item */
        return 1;
    }, $items);

    array_walk($items, function (&$i) {
        /** @mir-check $i is mixed */
    });

    array_filter($items, function ($i) {
        /** @mir-check $i is mixed */
        return true;
    }, $untyped[0]);
//     ^^^^^^^^^^^ MixedArgument: Argument $mode of array_filter() is mixed
}
