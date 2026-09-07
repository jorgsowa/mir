===description===
A plain (non-parameterized) @psalm-type alias never expanded when nested
INSIDE another type — as a generic type argument (`Box<IntList>`), or
inside another alias's own definition (`IntListList = array<IntList>`).
expand_aliases_only only ever substituted a top-level atom, never
recursing into type_params/array value-types, so both stayed unexpanded
(mixed) even though the bare, non-nested alias usage already worked.
===config===
suppress=UnusedParam,MixedAssignment,MissingPropertyType
===file===
<?php
/** @template T */
class Box {
    /** @var T */
    public $value;
}

/**
 * @psalm-type IntList = array<int>
 * @psalm-type IntListList = array<IntList>
 */
class Repo {
    /**
     * @param Box<IntList> $b
     */
    public function viaGenericArg(Box $b): void {
        foreach ($b->value as $x) {
            strlen($x);
        }
    }

    /**
     * @param IntListList $x
     */
    public function viaNestedAliasDefinition(array $x): void {
        foreach ($x as $inner) {
            foreach ($inner as $v) {
                strlen($v);
            }
        }
    }
}
===expect===
ArgumentTypeCoercion@18:19-18:21: Argument $string of strlen() expects 'string', got 'int' — coercion may fail at runtime
ArgumentTypeCoercion@28:23-28:25: Argument $string of strlen() expects 'string', got 'int' — coercion may fail at runtime
