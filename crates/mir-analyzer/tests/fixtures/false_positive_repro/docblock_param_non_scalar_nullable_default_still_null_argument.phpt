===description===
Same underlying gap, manifesting as a false NullArgument instead: an
untyped-param-with-null-default convention (`?array $ids = null`, "pass
null to get everything") paired with a non-nullable `@param list<string>`
docblock must still accept a literal `null` argument.
===file===
<?php
final class Service {
    /** @param list<string> $ids */
    public function getAll(?array $ids = null): array {
        return [];
    }
}
(new Service())->getAll(null);
===expect===
UnusedParam@4:27-4:45: Parameter $ids is never used
