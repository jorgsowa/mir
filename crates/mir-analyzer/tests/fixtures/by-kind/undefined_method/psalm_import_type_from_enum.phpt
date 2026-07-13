===description===
FN: @psalm-import-type only ever looked up the `from` target in
self.slice.classes — an interface/trait/enum declaring its own
@psalm-type alias could never be imported from.
===file===
<?php
class User {
    public function greet(): string { return ''; }
}

/**
 * @psalm-type UserType = User
 */
enum Repository {
    case Default;
}

/**
 * @psalm-import-type UserType from Repository
 * @method UserType find()
 */
class Service {}

function test(Service $s): void {
    $s->find()->greet();
    $s->find()->missing();
}
===expect===
UndefinedMethod@21:4-21:25: Method User::missing() does not exist
