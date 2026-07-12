===description===
KNOWN LIMITATION (not correct behavior, pinned so a future fix updates this
fixture deliberately instead of silently changing it): `inherited_template_bindings`
accumulates every @implements/@extends source's template bindings into one
flat map keyed only by the template parameter's bare name. When two
unrelated generic interfaces both happen to name their parameter `T` (an
extremely common convention), whichever is processed first wins and the
other is silently dropped — corrupting substitution for methods declared by
the loser. Here `Container<T>`'s binding for `get()`'s `T` is clobbered by
`Comparable<T>`'s binding, so `$result` resolves to `Foo` (Comparable's
argument) instead of the correct `Bar` (Container's own binding). A real fix
needs template bindings keyed by (name, defining_entity) — like
`Atomic::TTemplateParam` itself already carries — threaded through
`substitute_templates`, not just the bare name.

The same collision also makes `Container::get()`'s own `@return T` resolve
with a bare `T` left as a named-object reference instead of substituting to
`TTemplateParam`, which is why `UndefinedDocblockClass` now additionally
fires below — another symptom of this same pinned limitation, not a new bug.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @template T */
interface Comparable {
    /** @param T $other */
    public function compareTo($other): int;
}
/** @template T */
interface Container {
    /** @return T */
    public function get();
}

class Foo {}
class Bar {}

/**
 * @implements Comparable<Foo>
 * @implements Container<Bar>
 */
class Thing implements Comparable, Container {
    /** @param T $other */
    public function compareTo($other): int {
        return 0;
    }
    /** @return T */
    public function get() {
        return new Bar();
    }
}

$t = new Thing();
$result = $t->get();
/** @mir-check $result is Foo */
echo "ok";
===expect===
UndefinedDocblockClass@22:20-22:29: Docblock type 'T' does not exist
UndefinedDocblockClass@26:20-26:23: Docblock type 'T' does not exist
