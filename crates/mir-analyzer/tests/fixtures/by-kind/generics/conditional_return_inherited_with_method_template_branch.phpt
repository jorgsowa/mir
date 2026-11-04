===description===
An override inherits a conditional `@return` whose branches mention a method template when the template's bound fits its native return
===file===
<?php
interface Container {
    /**
     * @template C of object
     * @param string|class-string<C> $id
     * @return ($id is class-string<C> ? C : object)
     */
    public function get(string $id): ?object;
}
class Impl implements Container {
    /** Gets a service. */
    public function get(string $id): ?object { return new stdClass(); }
}
function f(Impl $c, string $id): object {
    return $c->get($id);
}
