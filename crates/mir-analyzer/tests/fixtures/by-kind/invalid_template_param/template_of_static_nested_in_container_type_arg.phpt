===description===
`@template U of Collection<static>` — the `static` placeholder is nested
inside Collection's own type arg, not top-level. `resolve_static_in_bound`
only rewrote a top-level `static` atom, so this always failed the bound
check (a false positive) even for a genuinely-typed receiver. A real
mismatch through the same nested position still correctly violates.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
/** @template T */
class Collection {
    /** @param T $item */
    public function __construct($item) {}
}

class Repo {
    /**
     * @template U of Collection<static>
     * @param U $items
     */
    public function tagged($items): void {}
}

class NotTaggableRepo extends Repo {}
class Unrelated {}

function acceptsOwnClass(NotTaggableRepo $r): void {
    $r->tagged(new Collection($r));
}

function rejectsUnrelatedClass(NotTaggableRepo $r, Unrelated $u): void {
    $r->tagged(new Collection($u));
}
===expect===
InvalidTemplateParam@24:4-24:34: Template type 'U' inferred as 'Collection<Unrelated>' does not satisfy bound 'Collection<NotTaggableRepo>'
