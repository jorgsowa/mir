===description===
template parameters mixed with concrete types should not cause InvalidArgument
===file===
<?php
class User { }
class Data { }

/**
 * @template T
 * @param T $data
 * @param User $user
 */
function saveUserData(mixed $data, User $user): void {}
//                    ^^^^^^^^^^^ UnusedParam: Parameter $data is never used
//                                 ^^^^^^^^^^ UnusedParam: Parameter $user is never used

function test(): void {
    // Mix of template and concrete types - should not report InvalidArgument
    saveUserData(new Data(), new User());
    saveUserData("string", new User());
    saveUserData(123, new User());
}
===expect===
