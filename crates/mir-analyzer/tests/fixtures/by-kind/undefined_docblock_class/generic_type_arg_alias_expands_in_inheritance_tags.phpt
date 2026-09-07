===description===
A local @psalm-type alias used as a generic type argument in @extends/
@implements/@use was never expanded -- every one of those four collector
sites (class.rs's extends/implements/trait_use, interface.rs's extends,
enum.rs's implements, trait.rs's trait_use) passed the raw alias atom
straight into resolve_union_doc_with_templates, unlike the sibling
@template bound/default handling right next to it which already expands
aliases first. Covers all four collector sites in one fixture since each
shares the identical fix shape. The alias is declared fresh in each
class-like's own docblock (aliases are scoped per declaring class-like in
this codebase, not file-global).
===config===
suppress=UnusedParam,UnusedVariable,MissingThrowsDocblock,MissingConstructor,MissingPropertyType
===file===
<?php
/** @template T */
interface Collection {}

/**
 * @psalm-type ItemShape = array{id: int, name: string}
 * @implements Collection<ItemShape>
 */
class ClassCase implements Collection {}

/** @template T */
interface CollectionBase {}

/**
 * @psalm-type ItemShape = array{id: int, name: string}
 * @extends CollectionBase<ItemShape>
 */
interface CollectionExtended extends CollectionBase {}

class InterfaceCase implements CollectionExtended {}

/**
 * @psalm-type ItemShape = array{id: int, name: string}
 * @implements Collection<ItemShape>
 */
enum EnumCase implements Collection {
    case Only;
}

/** @template T */
trait Container {
    /** @var T */
    protected $value;
    /** @return T */
    public function get() {
        return $this->value;
    }
}

/**
 * @psalm-type ItemShape = array{id: int, name: string}
 * @use Container<ItemShape>
 */
class TraitCase {
    use Container;
}

/**
 * @template X
 * @param Collection<X> $c
 * @return X
 */
function firstOf(Collection $c) {
    throw new \Exception();
}

/**
 * @template Y
 * @param CollectionBase<Y> $c
 * @return Y
 */
function baseOf(CollectionBase $c) {
    throw new \Exception();
}

function test(ClassCase $cc, InterfaceCase $ic, EnumCase $ec, TraitCase $tc): void {
    $x1 = firstOf($cc);
    /** @mir-check $x1 is array{id: int, name: string} */
    $_ = $x1;

    $x2 = baseOf($ic);
    /** @mir-check $x2 is array{id: int, name: string} */
    $_ = $x2;

    $x3 = firstOf($ec);
    /** @mir-check $x3 is array{id: int, name: string} */
    $_ = $x3;

    /** @mir-check $tc->get() is array{id: int, name: string} */
    $_ = 1;
}
===expect===
