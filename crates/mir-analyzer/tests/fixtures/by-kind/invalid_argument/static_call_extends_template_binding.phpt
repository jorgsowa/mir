===description===
Static method calls resolve class-level template bindings from @extends, like instance calls do
===file===
<?php
/** @template T */
abstract class Repository {
    /** @param T $item */
    public static function validate($item): void {}
//                                  ^^^^^ UnusedParam: Parameter $item is never used
}

class User {}
class Post {}

/** @extends Repository<User> */
class UserRepository extends Repository {}

UserRepository::validate(new Post());
//                       ^^^^^^^^^^ InvalidArgument: Argument $item of validate() expects 'User', got 'Post'
