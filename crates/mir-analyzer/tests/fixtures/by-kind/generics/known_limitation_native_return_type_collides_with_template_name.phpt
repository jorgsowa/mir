===description===
KNOWN LIMITATION (not correct behavior, pinned so a future fix updates this
fixture deliberately instead of silently changing it): `substitute_templates`
(mir-types/src/union.rs) treats any bare, unqualified `TNamedObject { fqcn,
type_params: [] }` as a template reference whenever `fqcn` is a key in the
active bindings map — with no way to tell that reference apart from a
genuine class of the same name. Here `makeRealT(): T` has NO `@return`
docblock at all; its return type comes purely from the native hint, which
PHP itself always resolves against the real, declared class `\T` — never
against `Box`'s unrelated `@template T`. But once `$box` is known to be
`Box<int>`, substituting that binding into every one of `Box`'s method
return types (to resolve `get()`'s own `@return T`) also corrupts
`makeRealT`'s unrelated, unambiguous native return type into `int`. A real
fix needs the docblock/hint parser to be template-aware enough to never
produce an ambiguous bare `TNamedObject` for a template reference in the
first place (see the TODO in `Type::substitute_templates`'s `TNamedObject`
arm) — the same underlying gap as
known_limitation_implements_template_name_collision, just reached through a
native type hint instead of a second `@implements` source.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
final class T {
    public int $x = 1;
}

/**
 * @template T
 */
final class Box {
    /** @param T $value */
    public function __construct(private $value) {}

    /** @return T */
    public function get() {
        return $this->value;
    }

    // No @return docblock — the native hint alone means the real class \T,
    // unrelated to Box's own template placeholder of the same name.
    public function makeRealT(): T {
        return new T();
    }
}

$box = new Box(42);
$real = $box->makeRealT();
/** @mir-check $real is T */
$v = $real->x;
===expect===
MixedAssignment@28:0-28:13: Variable $v is assigned a mixed type
TypeCheckMismatch@28:0-28:14: Type of $real is expected to be T, got int
InvalidPropertyFetch@28:5-28:13: Cannot fetch property on non-object type 'int'
