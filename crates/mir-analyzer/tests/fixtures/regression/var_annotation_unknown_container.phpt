===description===
Regression: @var annotation should narrow variable type even when RHS is from an unknown/mixed source
===file===
<?php
class UserRepository {
    public function find(int $id): ?object { return null; }
}
function main(mixed $container): void {
    /** @var UserRepository $repo */
    $repo = $container->get(UserRepository::class);
    /** @mir-check $repo is UserRepository */
    $repo->find(1);
}
===expect===
MixedMethodCall@7:13-7:51: Method get() called on mixed type
