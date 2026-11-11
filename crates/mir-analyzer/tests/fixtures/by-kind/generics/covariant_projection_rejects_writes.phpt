===description===
`Shelf<covariant Fruit>` may hold any subtype, so nothing can be written through it; reading yields the bound.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedProperty errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Fruit {}
final class Apple extends Fruit {}

/** @template T */
final class Shelf {
    /** @param T $item */
    public function __construct(private mixed $item) {}

    /** @param T $item */
    public function replace(mixed $item): void {
        $this->item = $item;
    }

    /** @return T */
    public function first(): mixed {
        return $this->item;
    }
}

function takesFruit(Fruit $fruit): void {}

/** @param Shelf<covariant Fruit> $shelf */
function restock(Shelf $shelf): void {
    $shelf->replace(new Apple());
//                  ^^^^^^^^^^^ InvalidArgument: Argument $item of replace() expects 'never', got 'Apple'
    $shelf->replace(new Fruit());
//                  ^^^^^^^^^^^ InvalidArgument: Argument $item of replace() expects 'never', got 'Fruit'
    takesFruit($shelf->first());
}
