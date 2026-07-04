===description===
Static method calls resolve class-level template bindings from @extends, like instance calls do
===file===
<?php
/** @template T */
abstract class Repository {
    /** @param T $item */
    public static function validate($item): void {}
}

class User {}
class Post {}

/** @extends Repository<User> */
class UserRepository extends Repository {}

UserRepository::validate(new Post());
===expect===
UnusedParam@5:36-5:41: Parameter $item is never used
InvalidArgument@14:25-14:35: Argument $item of validate() expects 'User', got 'Post'
