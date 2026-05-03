===description===
psalm import type resolves alias
===file===
<?php
class User {
    public function greet(): string { return ''; }
}

/**
 * @psalm-type UserType = User
 */
class Repository {}

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
UndefinedMethod@19:4: Method User::missing() does not exist
===ignore===
TODO
