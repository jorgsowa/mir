===description===
G1: an `@return T of Animal` method returns a value that is not even a subtype of the
template bound (a Plant) — the erased return type is `Animal`, so this is a real
InvalidReturnType.
===config===
suppress=UnusedParam
===file===
<?php
class Animal {}
class Plant {}

/**
 * @template T of Animal
 */
class Crate {
    /** @return T */
    public function bad() {
        return new Plant();
    }
}
===expect===
InvalidReturnType@11:8-11:27: Return type 'Plant' is not compatible with declared 'T'
