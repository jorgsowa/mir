===description===
FN: a `@template T of Ancestor<Bound>` check silently passed when `Ancestor`
was reachable only through an interface's OWN `@extends` (two hops away from
the concrete class), for the same root cause as the sibling InvalidArgument
gap: `InterfaceDef` didn't carry `@extends` type args, so the inherited-
binding walk couldn't resolve the distant ancestor's template param and fell
back to `mixed`, which trivially satisfies any bound.
===config===
suppress=MissingPropertyType,UnusedParam
===file===
<?php
/** @template-covariant E */
interface GrandCollection {}

/**
 * @template-covariant T
 * @extends GrandCollection<T>
 */
interface Collection extends GrandCollection {}

class Animal {}
class Unrelated {}

/**
 * @template T
 * @implements Collection<T>
 */
class TypedList implements Collection {
    /** @param T $item */
    public function __construct(private $item) {}
}

/**
 * @template T of GrandCollection<Animal>
 * @param T $c
 */
function accept_bad($c): void {}

accept_bad(new TypedList(new Unrelated()));
===expect===
InvalidTemplateParam@29:0-29:42: Template type 'T' inferred as 'TypedList<Unrelated>' does not satisfy bound 'GrandCollection<Animal>'
