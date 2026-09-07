===description===
`parse_generic`'s catch-all had no special case for `self`/`static`/
`parent` as the generic base name, unlike the bare-word path — `@return
self<T>` (a common Doctrine/Collection-style "generic self-referencing
collection" pattern) parsed as a bogus `TNamedObject` named `"self"`
instead of `TSelf`, an unresolvable pseudo-class that collapsed method
resolution on it to `mixed`. The written `<T>` is dropped (`TSelf`
carries no `type_params` of its own), matching how a bare `@return self`
already resolves via the receiver's own concrete binding.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
/** @template T */
class Collection {
    /** @return self<T> */
    public function identity(): self {
        return $this;
    }
}

class IntCollection extends Collection {}

/** @param IntCollection<int> $c */
function test($c): void {
    $result = $c->identity();
    /** @mir-check $result is IntCollection<int> */
    $_ = 1;
}
===expect===
