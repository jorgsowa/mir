===description===
Regression (laravel/framework): a bare `return;` yields null, which is assignable
to a nullable declared return type (?User). The bare-return guard now also accepts
a declared type that allows null or includes void in a union, so no
InvalidReturnType.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class User {}
class EloquentUserProvider {
    public function retrieveByToken(int $id): ?User {
        if ($id < 0) {
            return;
        }
        return new User();
    }
}
