===description===
class level template resolves to concrete type
===file===
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
UndefinedMethod@14:5-14:32: Method User::nonExistentMethod() does not exist
