===source===
<?php
/** @template T */
class Collection {
    /** @return T */
    public function first(): mixed { return null; }
}
class User {
    public function getName(): string { return 'Alice'; }
}
function test(): void {
    /** @var Collection<User> $items */
    $items = new Collection();
    $first = $items->first();
    $first->nonExistentMethod();
}
===expect===
UndefinedMethod: $first->nonExistentMethod()
