===description===
Local type aliases that mention each other expand in either declaration order, and a list of the resulting shape is checked.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @psalm-type UserId = int
 * @psalm-type User = array{id: UserId, name: string}
 * @psalm-type UserList = list<User>
 */
final class Forward
{
    /** @param UserList $users */
    public function take(array $users): void {}
}

/**
 * @phpstan-type ReverseList list<ReverseUser>
 * @phpstan-type ReverseUser array{id: ReverseId, name: string}
 * @phpstan-type ReverseId int
 */
final class Reverse
{
    /** @param ReverseList $users */
    public function take(array $users): void {}
}

(new Forward())->take([['id' => 1, 'name' => 'a']]);
(new Forward())->take([['id' => 'x', 'name' => 'a']]);
//                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $users of take() expects 'list<array{'id': int, 'name': string}>', got 'array{0: array{'id': "x", 'name': "a"}}'
(new Reverse())->take([['id' => 1, 'name' => 'a']]);
(new Reverse())->take([['id' => 'x', 'name' => 'a']]);
//                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $users of take() expects 'list<array{'id': int, 'name': string}>', got 'array{0: array{'id': "x", 'name': "a"}}'
