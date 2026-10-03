===description===
Regression: @var annotation should narrow variable type even when RHS is from an unknown/mixed source
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class UserRepository {
    public function find(int $id): ?object { return null; }
}
function main(mixed $container): void {
    /** @var UserRepository $repo */
    $repo = $container->get(UserRepository::class);
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MixedMethodCall: Method get() called on mixed type
    /** @mir-check $repo is UserRepository */
    $repo->find(1);
}
===expect===
