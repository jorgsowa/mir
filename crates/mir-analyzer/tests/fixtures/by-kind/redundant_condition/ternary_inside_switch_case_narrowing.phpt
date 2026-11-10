===description===
A ternary inside a `switch (get_class())` or `switch (gettype())` case is checked against the type that case narrowed to.
===file===
<?php
final class Cat {}
final class Dog {}

function byClass(Cat|Dog $pet): string {
    switch (get_class($pet)) {
        case Cat::class:
            return $pet instanceof Dog ? 'dog' : 'cat';
//                 ^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
    }
    return $pet instanceof Dog ? 'dog' : 'cat';
}

function byGettype(int|string $id): string {
    switch (gettype($id)) {
        case 'integer':
            return is_string($id) ? 'string' : 'int';
//                 ^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
    }
    return is_string($id) ? 'string' : 'int';
}
