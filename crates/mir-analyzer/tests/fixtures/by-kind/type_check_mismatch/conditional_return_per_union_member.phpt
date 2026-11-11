===description===
A conditional return on a class resolves each member of a union argument on its own: `?User` for `($key is Model ? A : B)` takes `A` for `User` and `B` for `null`. A `null` or array argument is resolved against a subject mixing classes and `array`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

interface Arrayable {}
class Model implements Arrayable {}
final class User extends Model {}

/** @return ($key is Model ? int : ($key is (Arrayable|array<mixed>) ? float : string)) */
function find(mixed $key): mixed { return 1; }

function lookups(User $user, ?User $maybeUser, mixed $any): void {
    $a = find($user);
    /** @mir-check $a is int */
    $b = find($maybeUser);
    /** @mir-check $b is int|string */
    $c = find(null);
    /** @mir-check $c is string */
    $d = find([1]);
    /** @mir-check $d is float */
    $e = find($any);
    /** @mir-check $e is int|float|string */
    echo 1;
}
