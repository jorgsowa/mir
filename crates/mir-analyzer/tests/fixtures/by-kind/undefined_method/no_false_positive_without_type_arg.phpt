===description===
no false positive without type arg
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
class BaseRepo {
    /** @return T */
    public function find(): mixed { return null; }
}
class UserRepo extends BaseRepo {}

$repo = new UserRepo();
$result = $repo->find();
