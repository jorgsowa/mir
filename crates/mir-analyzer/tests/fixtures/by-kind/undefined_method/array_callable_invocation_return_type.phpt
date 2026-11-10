===description===
Invoking an array callable `[$object, 'method']` or `[Foo::class, 'method']`
returns that method's return type, with `static` bound to the target class.
===file===
<?php
final class User {
    public function name(): string { return 'user'; }
}

class Repository {
    public function find(): User { return new User(); }
    public static function make(): static { return new static(); }
}

function run(Repository $repository): void {
    $find = [$repository, 'find'];
    echo $find()->name();
    echo $find()->missing();
//       ^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method User::missing() does not exist
    $make = [Repository::class, 'make'];
    /** @mir-check $make() is Repository */
    echo $make()->find()->name();
}
