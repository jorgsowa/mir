===description===
Regression: @var ClassName $var docblock annotation should narrow variable type after assignment
===config===
suppress=MixedAssignment,UnusedParam
===file===
<?php
class UserRepository {
    public function find(int $id): ?object { return null; }
}
class Container {
    public function get(string $class): mixed { return null; }
}
$container = new Container();
/** @var UserRepository $repo */
$repo = $container->get(UserRepository::class);
/** @mir-check $repo is UserRepository */
$repo->find(1);
===expect===
