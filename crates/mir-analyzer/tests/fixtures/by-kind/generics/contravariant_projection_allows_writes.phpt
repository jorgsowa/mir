===description===
`Bin<contravariant Apple>` accepts writes, and a method with its own template of the same name is not affected by a star projection.
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
final class Bin {
    /** @param T $item */
    public function add(mixed $item): void {}

    /**
     * @template T
     * @param T $value
     * @return T
     */
    public function echoBack(mixed $value): mixed {
        return $value;
    }
}

/** @param Bin<contravariant Apple> $bin */
function collect(Bin $bin): void {
    $bin->add(new Apple());
}

/** @param Bin<*> $bin */
function probe(Bin $bin): int {
    return $bin->echoBack(5);
//         ^^^^^^^^^^^^^^^^^ ShadowedTemplateParam: Method template parameter 'T' shadows class-level template parameter with the same name
}
