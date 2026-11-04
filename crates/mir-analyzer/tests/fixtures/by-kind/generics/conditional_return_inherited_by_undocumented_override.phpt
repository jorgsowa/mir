===description===
An override without `@return` inherits the parent's conditional `@return` when its branches fit the native return
===file===
<?php
interface Container {
    /** @return ($b is true ? object : object|null) */
    public function get(string $id, bool $b): ?object;
}
class Impl implements Container {
    /** Gets a service. */
    public function get(string $id, bool $b): ?object { return $b ? new stdClass() : null; }
}
function f(Impl $c): object {
    return $c->get('x', true);
}
